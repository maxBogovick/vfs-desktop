use std::sync::{Arc, RwLock, Mutex};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use chrono::Utc;
use once_cell::sync::Lazy;
use tracing::{info, warn, error, debug};

use super::types::*;
use super::persistence::QueuePersistence;
use super::executor::OperationExecutor;
use super::scheduler::TaskScheduler;

/// Wrapper for prioritizing operations in the queue
#[derive(Debug, Clone)]
struct PrioritizedOperation {
    operation: QueuedOperation,
}

impl PartialEq for PrioritizedOperation {
    fn eq(&self, other: &Self) -> bool {
        self.operation.id == other.operation.id
    }
}

impl Eq for PrioritizedOperation {}

impl PartialOrd for PrioritizedOperation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedOperation {
    fn cmp(&self, other: &Self) -> Ordering {
        // First by priority (higher = higher priority)
        match self.operation.priority.cmp(&other.operation.priority) {
            Ordering::Equal => {
                // Then by creation time (earlier = higher priority)
                // Reverse ordering so earlier time > later time
                other.operation.created_at.cmp(&self.operation.created_at)
            }
            ordering => ordering,
        }
    }
}

/// Queue manager for operations
pub struct QueueManager {
    /// All operations by ID
    operations: Arc<RwLock<HashMap<String, QueuedOperation>>>,

    /// Priority queue for execution
    priority_queue: Arc<RwLock<BinaryHeap<PrioritizedOperation>>>,

    /// Currently running operation IDs
    running_operations: Arc<RwLock<Vec<String>>>,

    /// Configuration
    config: Arc<RwLock<QueueConfig>>,

    /// Persistence handler
    persistence: Arc<QueuePersistence>,

    /// Task scheduler
    scheduler: Arc<TaskScheduler>,

    /// Operation executor
    executor: Arc<OperationExecutor>,

    /// Background task handle
    background_task: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl QueueManager {
    /// Create a new queue manager
    pub fn new() -> Self {
        let config = QueueConfig::default();
        let persistence = Arc::new(QueuePersistence::new());

        // Load persisted operations
        let (operations, priority_queue) = match persistence.load() {
            Ok(ops) => {
                info!("Loaded {} operations from persistence", ops.len());
                let mut operations_map = HashMap::new();
                let mut pq = BinaryHeap::new();

                for mut op in ops {
                    // Only restore queued and scheduled operations
                    // Running operations are marked as failed (interrupted)
                    match op.status {
                        QueueOperationStatus::Running | QueueOperationStatus::Paused => {
                            warn!("Operation {} was interrupted, marking as failed", op.id);
                            op.mark_failed("Operation interrupted by application shutdown".to_string());
                        }
                        QueueOperationStatus::Queued => {
                            pq.push(PrioritizedOperation { operation: op.clone() });
                        }
                        QueueOperationStatus::Scheduled => {
                            // Will be moved to queue by scheduler
                        }
                        _ => {}
                    }
                    operations_map.insert(op.id.clone(), op);
                }

                (operations_map, pq)
            }
            Err(e) => {
                warn!("Failed to load operations from persistence: {}", e);
                (HashMap::new(), BinaryHeap::new())
            }
        };

        let manager = Self {
            operations: Arc::new(RwLock::new(operations)),
            priority_queue: Arc::new(RwLock::new(priority_queue)),
            running_operations: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(RwLock::new(config)),
            persistence,
            scheduler: Arc::new(TaskScheduler::new()),
            executor: Arc::new(OperationExecutor::new()),
            background_task: Arc::new(Mutex::new(None)),
        };

        // Start background task for scheduler
        manager.start_background_scheduler();

        manager
    }

    /// Add operation to queue
    pub fn enqueue(&self, mut operation: QueuedOperation) -> Result<String, String> {
        let operation_id = operation.id.clone();
        info!("Enqueueing operation: {} ({:?})", operation_id, operation.operation_type);

        // If scheduled_at is set, mark as Scheduled
        if operation.scheduled_at.is_some() {
            operation.status = QueueOperationStatus::Scheduled;
        }

        // Add to operations map
        {
            let mut ops = self.operations.write().unwrap();
            ops.insert(operation_id.clone(), operation.clone());
        }

        // Add to priority queue if not scheduled
        if operation.scheduled_at.is_none() {
            let mut pq = self.priority_queue.write().unwrap();
            pq.push(PrioritizedOperation { operation });
        }

        // Persist
        self.persist_if_enabled();

        // Auto-start if enabled
        let config = self.config.read().unwrap();
        if config.auto_start {
            drop(config);
            self.process_queue();
        }

        Ok(operation_id)
    }

    /// Process queue (start executing operations)
    pub fn process_queue(&self) {
        let max_parallel = self.config.read().unwrap().max_parallel_operations;

        loop {
            // Check how many are running
            let running_count = self.running_operations.read().unwrap().len();
            if running_count >= max_parallel {
                debug!("Max parallel operations reached ({}), waiting...", max_parallel);
                break;
            }

            // Get next operation
            let next_op = {
                let mut pq = self.priority_queue.write().unwrap();
                pq.pop()
            };

            match next_op {
                Some(prioritized_op) => {
                    let operation = prioritized_op.operation;
                    let op_id = operation.id.clone();

                    // Check if operation is still queued (handle duplicates in PQ or cancelled ops)
                    {
                        let ops = self.operations.read().unwrap();
                        if let Some(current_op) = ops.get(&op_id) {
                            if current_op.status != QueueOperationStatus::Queued {
                                debug!("Skipping operation {} as it is not Queued (status: {:?})", op_id, current_op.status);
                                continue;
                            }
                        } else {
                            debug!("Skipping operation {} as it no longer exists", op_id);
                            continue;
                        }
                    }

                    // Mark as running
                    {
                        let mut running = self.running_operations.write().unwrap();
                        running.push(op_id.clone());
                    }

                    // Update operation status
                    {
                        let mut ops = self.operations.write().unwrap();
                        if let Some(op) = ops.get_mut(&op_id) {
                            op.mark_started();
                        }
                    }
                    self.persist_if_enabled();

                    // Execute in background thread
                    let manager = self.clone();
                    std::thread::spawn(move || {
                        manager.execute_operation(operation);
                    });
                }
                None => {
                    debug!("Queue is empty");
                    break;
                }
            }
        }
    }

    /// Execute a single operation
    fn execute_operation(&self, mut operation: QueuedOperation) {
        let op_id = operation.id.clone();
        info!("Executing operation: {} ({:?})", op_id, operation.operation_type);

        // Execute through executor
        let result = self.executor.execute(&operation);

        // Update operation status
        {
            let mut ops = self.operations.write().unwrap();
            if let Some(op) = ops.get_mut(&op_id) {
                // If operation was cancelled while running, don't overwrite the status
                if op.status == QueueOperationStatus::Cancelled {
                    info!("Operation {} was cancelled while running", op_id);
                } else {
                    match result {
                        Ok(_) => {
                            op.mark_completed();
                            info!("Operation completed: {}", op_id);
                        }
                        Err(e) => {
                            error!("Operation failed: {} - {}", op_id, e);
                            op.mark_failed(e.clone());

                            // If should retry, add back to queue
                            if op.status == QueueOperationStatus::Scheduled {
                                // Retry is scheduled
                                info!("Operation {} will retry at {:?}", op_id, op.scheduled_at);
                            }
                        }
                    }
                }
            }
        }

        // Remove from running
        {
            let mut running = self.running_operations.write().unwrap();
            running.retain(|id| id != &op_id);
        }

        // Persist
        self.persist_if_enabled();

        // Continue processing queue
        self.process_queue();
    }

    /// Cancel an operation
    pub fn cancel_operation(&self, operation_id: &str) -> Result<(), String> {
        info!("Cancelling operation: {}", operation_id);

        // Check if running
        let is_running = {
            let running = self.running_operations.read().unwrap();
            running.contains(&operation_id.to_string())
        };

        if is_running {
            // Cancel through ProgressTracker
            crate::progress::OPERATIONS_MANAGER.cancel_operation(operation_id);
        }

        // Update status
        let mut ops = self.operations.write().unwrap();
        if let Some(op) = ops.get_mut(operation_id) {
            op.mark_cancelled();
            drop(ops);
            self.persist_if_enabled();
            Ok(())
        } else {
            Err(format!("Operation not found: {}", operation_id))
        }
    }

    /// Retry a failed operation manually
    pub fn retry_operation(&self, operation_id: &str) -> Result<(), String> {
        info!("Retrying operation: {}", operation_id);

        let mut ops = self.operations.write().unwrap();
        if let Some(op) = ops.get_mut(operation_id) {
            if op.status != QueueOperationStatus::Failed && op.status != QueueOperationStatus::Cancelled {
                return Err("Operation is not in failed or cancelled state".to_string());
            }

            // Reset operation
            op.status = QueueOperationStatus::Queued;
            op.current_attempt = 0;
            op.error_message = None;
            op.scheduled_at = None;
            op.started_at = None;
            op.completed_at = None;

            // Add back to queue
            let mut pq = self.priority_queue.write().unwrap();
            pq.push(PrioritizedOperation { operation: op.clone() });

            drop(ops);
            drop(pq);
            self.persist_if_enabled();
            self.process_queue();

            Ok(())
        } else {
            Err(format!("Operation not found: {}", operation_id))
        }
    }

    /// Get all operations
    pub fn get_all_operations(&self) -> Vec<QueuedOperation> {
        let ops = self.operations.read().unwrap();
        ops.values().cloned().collect()
    }

    /// Get operation by ID
    pub fn get_operation(&self, operation_id: &str) -> Option<QueuedOperation> {
        let ops = self.operations.read().unwrap();
        ops.get(operation_id).cloned()
    }

    /// Remove an operation (only completed/cancelled/failed)
    pub fn remove_operation(&self, operation_id: &str) -> Result<(), String> {
        let mut ops = self.operations.write().unwrap();
        if let Some(op) = ops.get(operation_id) {
            if !matches!(
                op.status,
                QueueOperationStatus::Completed
                    | QueueOperationStatus::Cancelled
                    | QueueOperationStatus::Failed
            ) {
                return Err("Cannot remove active operation".to_string());
            }
        }

        ops.remove(operation_id);
        drop(ops);
        self.persist_if_enabled();
        Ok(())
    }

    /// Update queue configuration
    pub fn update_config(&self, new_config: QueueConfig) {
        let mut config = self.config.write().unwrap();
        *config = new_config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> QueueConfig {
        self.config.read().unwrap().clone()
    }

    /// Pause an operation
    pub fn pause_operation(&self, operation_id: &str) -> Result<(), String> {
        crate::progress::OPERATIONS_MANAGER.pause_operation(operation_id);
        Ok(())
    }

    /// Resume an operation
    pub fn resume_operation(&self, operation_id: &str) -> Result<(), String> {
        crate::progress::OPERATIONS_MANAGER.resume_operation(operation_id);
        Ok(())
    }

    /// Run an operation immediately (move to front of queue with urgent priority)
    pub fn run_now(&self, operation_id: &str) -> Result<(), String> {
        info!("Running operation immediately: {}", operation_id);

        let mut ops = self.operations.write().unwrap();
        if let Some(op) = ops.get_mut(operation_id) {
            // Only allow for queued or scheduled operations
            if !matches!(
                op.status,
                QueueOperationStatus::Queued | QueueOperationStatus::Scheduled
            ) {
                return Err("Operation must be queued or scheduled".to_string());
            }

            // Set to urgent priority and queued status
            op.priority = OperationPriority::Urgent;
            op.status = QueueOperationStatus::Queued;
            op.scheduled_at = None;

            // Add to priority queue (will be at front due to Urgent priority)
            let mut pq = self.priority_queue.write().unwrap();
            pq.push(PrioritizedOperation { operation: op.clone() });

            drop(ops);
            drop(pq);
            self.persist_if_enabled();
            self.process_queue();

            Ok(())
        } else {
            Err(format!("Operation not found: {}", operation_id))
        }
    }

    /// Persist queue if enabled
    fn persist_if_enabled(&self) {
        let config = self.config.read().unwrap();
        if config.persist_on_change {
            drop(config);
            let ops = self.get_all_operations();
            if let Err(e) = self.persistence.save(&ops) {
                error!("Failed to persist queue: {}", e);
            }
        }
    }

    /// Start background scheduler task
    fn start_background_scheduler(&self) {
        let manager = self.clone();
        let check_interval = self.config.read().unwrap().check_scheduled_interval_sec;

        let handle = std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(check_interval));
                manager.check_scheduled_operations();
            }
        });

        *self.background_task.lock().unwrap() = Some(handle);
    }

    /// Check scheduled operations and move to queue if ready
    fn check_scheduled_operations(&self) {
        let now = Utc::now();
        let mut ops = self.operations.write().unwrap();
        let mut to_queue = Vec::new();

        for op in ops.values_mut() {
            if op.status == QueueOperationStatus::Scheduled {
                if let Some(scheduled_at) = op.scheduled_at {
                    if self.scheduler.is_time_to_execute(scheduled_at) {
                        op.status = QueueOperationStatus::Queued;
                        op.scheduled_at = None;
                        to_queue.push(op.clone());
                    }
                }
            }
        }

        drop(ops);

        // Add to priority queue
        if !to_queue.is_empty() {
            let mut pq = self.priority_queue.write().unwrap();
            for op in to_queue {
                info!("Scheduled operation {} is now queued", op.id);
                pq.push(PrioritizedOperation { operation: op });
            }
            drop(pq);
            self.persist_if_enabled();
            self.process_queue();
        }
    }
}

// Implement Clone for Arc-based sharing
impl Clone for QueueManager {
    fn clone(&self) -> Self {
        Self {
            operations: Arc::clone(&self.operations),
            priority_queue: Arc::clone(&self.priority_queue),
            running_operations: Arc::clone(&self.running_operations),
            config: Arc::clone(&self.config),
            persistence: Arc::clone(&self.persistence),
            scheduler: Arc::clone(&self.scheduler),
            executor: Arc::clone(&self.executor),
            background_task: Arc::clone(&self.background_task),
        }
    }
}

/// Global queue manager instance
pub static QUEUE_MANAGER: Lazy<Arc<QueueManager>> = Lazy::new(|| {
    info!("Initializing global queue manager");
    Arc::new(QueueManager::new())
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        let op1 = QueuedOperation::new(
            QueuedOperationType::Copy,
            OperationParams::Copy {
                sources: vec![],
                destination: String::new(),
                source_fs: None,
                dest_fs: None,
            },
            OperationPriority::Low,
        );

        let op2 = QueuedOperation::new(
            QueuedOperationType::Copy,
            OperationParams::Copy {
                sources: vec![],
                destination: String::new(),
                source_fs: None,
                dest_fs: None,
            },
            OperationPriority::Urgent,
        );

        let p1 = PrioritizedOperation { operation: op1 };
        let p2 = PrioritizedOperation { operation: op2 };

        // Urgent should come before Low
        assert!(p2 > p1);
    }

    use std::collections::HashMap;

    #[test]
    fn test_queue_lifecycle() {
        // 0. Disable auto-start to avoid race conditions
        let mut config = QUEUE_MANAGER.get_config();
        config.auto_start = false;
        QUEUE_MANAGER.update_config(config);

        // 1. Create a dummy operation
        let op = QueuedOperation::new(
            QueuedOperationType::Custom("test_op".to_string()),
            OperationParams::Custom {
                command: "echo test".to_string(),
                args: HashMap::new(),
            },
            OperationPriority::Normal,
        );

        // 2. Add to queue
        let id = QUEUE_MANAGER.enqueue(op).expect("Failed to enqueue");
        println!("Enqueued operation: {}", id);

        // 3. Verify it exists
        let fetched_op = QUEUE_MANAGER.get_operation(&id).expect("Operation not found");
        assert_eq!(fetched_op.id, id);
        assert_eq!(fetched_op.status, QueueOperationStatus::Queued);

        // 4. Cancel it
        QUEUE_MANAGER.cancel_operation(&id).expect("Failed to cancel");

        // 5. Verify status
        let cancelled_op = QUEUE_MANAGER.get_operation(&id).unwrap();
        assert_eq!(cancelled_op.status, QueueOperationStatus::Cancelled);

        // 6. Retry it
        QUEUE_MANAGER.retry_operation(&id).expect("Failed to retry");

        // 7. Verify status back to Queued, Running, or Scheduled (if it ran and failed and scheduled retry)
        let retried_op = QUEUE_MANAGER.get_operation(&id).unwrap();
        assert!(
            retried_op.status == QueueOperationStatus::Queued ||
                retried_op.status == QueueOperationStatus::Running ||
                retried_op.status == QueueOperationStatus::Scheduled
        );

        // 8. Remove it
        // Must be cancelled/failed/completed to remove. Since it's Queued/Running/Scheduled now, we can't remove directly.
        // Let's cancel it again
        QUEUE_MANAGER.cancel_operation(&id).expect("Failed to cancel for remove");

        // Wait a tiny bit to ensure status persistence if any async stuff is happening (though cancel should be atomic on RwLock)
        std::thread::sleep(std::time::Duration::from_millis(50));

        QUEUE_MANAGER.remove_operation(&id).expect("Failed to remove");

        // 9. Verify gone
        assert!(QUEUE_MANAGER.get_operation(&id).is_none());
    }

    #[test]
    fn test_queue_run_now() {
        // 0. Ensure clean state
        let mut config = QUEUE_MANAGER.get_config();
        config.auto_start = false;
        QUEUE_MANAGER.update_config(config);

        // 1. Create operation
        let op = QueuedOperation::new(
            QueuedOperationType::Custom("run_now_op".to_string()),
            OperationParams::Custom {
                command: "echo test".to_string(),
                args: HashMap::new(),
            },
            OperationPriority::Low,
        );
        let id = QUEUE_MANAGER.enqueue(op).expect("Failed to enqueue");

        // 2. Call run_now
        QUEUE_MANAGER.run_now(&id).expect("Failed to run_now");

        // 3. Verify status changed to Queued (it was already Queued, but run_now resets it and changes priority)
        // and process_queue should have picked it up (since we are not mocking process_queue, it runs)
        // Wait a bit for background thread
        std::thread::sleep(std::time::Duration::from_millis(100));

        let op_after = QUEUE_MANAGER.get_operation(&id).unwrap();
        println!("Status after run_now: {:?}", op_after.status);

        // It should be Running or Scheduled (if it failed fast) or Completed (if it succeeded fast)
        // Since Custom op fails, it should be Scheduled (retry) or Failed (if retries exhausted/disabled)
        assert!(
            op_after.status == QueueOperationStatus::Running ||
                op_after.status == QueueOperationStatus::Scheduled ||
                op_after.status == QueueOperationStatus::Failed
        );

        // Verify priority was updated to Urgent
        assert_eq!(op_after.priority, OperationPriority::Urgent);

        // Cleanup
        let _ = QUEUE_MANAGER.cancel_operation(&id);
        let _ = QUEUE_MANAGER.remove_operation(&id);
    }
}
