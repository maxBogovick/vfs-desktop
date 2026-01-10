use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Extended operation types for the queue system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueuedOperationType {
    Copy,
    Move,
    Delete,
    Archive,
    Extract,
    BatchRename,
    BatchAttribute,
    Custom(String),
}

/// Operation status in the queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueueOperationStatus {
    /// In queue, waiting to be executed
    Queued,
    /// Scheduled for future execution
    Scheduled,
    /// Currently executing
    Running,
    /// Paused by user
    Paused,
    /// Successfully completed
    Completed,
    /// Failed after all retries
    Failed,
    /// Cancelled by user
    Cancelled,
}

/// Operation priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum OperationPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
    /// Whether retry is enabled
    pub enabled: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,      // 1 second
            max_delay_ms: 60000,          // 1 minute
            multiplier: 2.0,              // Exponential growth
            enabled: true,
        }
    }
}

/// Information about a single retry attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryAttempt {
    pub attempt_number: u32,
    pub timestamp: DateTime<Utc>,
    pub error_message: String,
    pub next_retry_at: Option<DateTime<Utc>>,
}

/// Operation parameters (polymorphic data)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum OperationParams {
    #[serde(rename_all = "camelCase")]
    Copy {
        sources: Vec<String>,
        destination: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_fs: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dest_fs: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Move {
        sources: Vec<String>,
        destination: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_fs: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dest_fs: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Delete {
        paths: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        panel_fs: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_fs: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Archive {
        sources: Vec<String>,
        archive_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_fs: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dest_fs: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Extract {
        archive_path: String,
        destination: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_fs: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dest_fs: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    BatchRename {
        items: Vec<String>,
        config: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_fs: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    BatchAttribute {
        items: Vec<String>,
        config: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        source_fs: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Custom {
        command: String,
        args: HashMap<String, serde_json::Value>,
    },
}

/// A queued operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedOperation {
    /// Unique operation ID
    pub id: String,
    /// Type of operation
    pub operation_type: QueuedOperationType,
    /// Current status
    pub status: QueueOperationStatus,
    /// Priority level
    pub priority: OperationPriority,
    /// Operation parameters
    pub params: OperationParams,

    // Metadata
    /// When the operation was created
    pub created_at: DateTime<Utc>,
    /// When the operation is scheduled to run (for scheduled tasks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<DateTime<Utc>>,
    /// When the operation started executing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    /// When the operation completed (success or failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,

    // Progress tracking
    /// ID of associated ProgressTracker (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_tracker_id: Option<String>,

    // Retry mechanism
    /// Retry policy for this operation
    pub retry_policy: RetryPolicy,
    /// History of retry attempts
    #[serde(default)]
    pub retry_attempts: Vec<RetryAttempt>,
    /// Current attempt number (0 = first attempt)
    #[serde(default)]
    pub current_attempt: u32,

    // Results
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// History of all error messages
    #[serde(default)]
    pub error_logs: Vec<String>,

    // User metadata
    /// Custom tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl QueuedOperation {
    /// Create a new queued operation
    pub fn new(
        operation_type: QueuedOperationType,
        params: OperationParams,
        priority: OperationPriority,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            operation_type,
            status: QueueOperationStatus::Queued,
            priority,
            params,
            created_at: Utc::now(),
            scheduled_at: None,
            started_at: None,
            completed_at: None,
            progress_tracker_id: None,
            retry_policy: RetryPolicy::default(),
            retry_attempts: Vec::new(),
            current_attempt: 0,
            error_message: None,
            error_logs: Vec::new(),
            tags: Vec::new(),
            description: None,
        }
    }

    /// Check if the operation should be retried
    pub fn should_retry(&self) -> bool {
        self.retry_policy.enabled
            && self.current_attempt < self.retry_policy.max_attempts
    }

    /// Calculate the next retry delay in milliseconds
    pub fn calculate_next_retry_delay(&self) -> u64 {
        let delay = (self.retry_policy.initial_delay_ms as f64)
            * self.retry_policy.multiplier.powi(self.current_attempt as i32);
        delay.min(self.retry_policy.max_delay_ms as f64) as u64
    }

    /// Mark operation as started
    pub fn mark_started(&mut self) {
        self.status = QueueOperationStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Mark operation as completed
    pub fn mark_completed(&mut self) {
        self.status = QueueOperationStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.error_message = None;
    }

    /// Mark operation as failed
    pub fn mark_failed(&mut self, error_msg: String) {
        self.error_message = Some(error_msg.clone());
        self.error_logs.push(error_msg.clone());

        if self.should_retry() {
            let delay_ms = self.calculate_next_retry_delay();
            let next_retry = Utc::now() + chrono::Duration::milliseconds(delay_ms as i64);

            self.retry_attempts.push(RetryAttempt {
                attempt_number: self.current_attempt,
                timestamp: Utc::now(),
                error_message: error_msg,
                next_retry_at: Some(next_retry),
            });

            self.status = QueueOperationStatus::Scheduled;
            self.scheduled_at = Some(next_retry);
            self.current_attempt += 1;
        } else {
            self.status = QueueOperationStatus::Failed;
            self.completed_at = Some(Utc::now());
        }
    }

    /// Mark operation as cancelled
    pub fn mark_cancelled(&mut self) {
        self.status = QueueOperationStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
}

/// Queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueConfig {
    /// Maximum number of operations running in parallel
    pub max_parallel_operations: usize,
    /// Automatically start processing queue when operation is added
    pub auto_start: bool,
    /// Save queue to disk on every change
    pub persist_on_change: bool,
    /// Interval in seconds to check for scheduled tasks
    pub check_scheduled_interval_sec: u64,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_parallel_operations: 3,
            auto_start: true,
            persist_on_change: true,
            check_scheduled_interval_sec: 60,  // Check every minute
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_creation() {
        let params = OperationParams::Copy {
            sources: vec!["/test/file".to_string()],
            destination: "/test/dest".to_string(),
            source_fs: None,
            dest_fs: None,
        };

        let op = QueuedOperation::new(
            QueuedOperationType::Copy,
            params,
            OperationPriority::Normal,
        );

        assert_eq!(op.status, QueueOperationStatus::Queued);
        assert_eq!(op.priority, OperationPriority::Normal);
        assert_eq!(op.current_attempt, 0);
        assert!(op.should_retry());
    }

    #[test]
    fn test_retry_delay_calculation() {
        let mut op = QueuedOperation::new(
            QueuedOperationType::Copy,
            OperationParams::Copy {
                sources: vec![],
                destination: String::new(),
                source_fs: None,
                dest_fs: None,
            },
            OperationPriority::Normal,
        );

        // First retry: 1000ms
        assert_eq!(op.calculate_next_retry_delay(), 1000);

        op.current_attempt = 1;
        // Second retry: 2000ms (1000 * 2^1)
        assert_eq!(op.calculate_next_retry_delay(), 2000);

        op.current_attempt = 2;
        // Third retry: 4000ms (1000 * 2^2)
        assert_eq!(op.calculate_next_retry_delay(), 4000);
    }

    #[test]
    fn test_serialization() {
        let params = OperationParams::Copy {
            sources: vec!["/test".to_string()],
            destination: "/dest".to_string(),
            source_fs: None,
            dest_fs: None,
        };

        let op = QueuedOperation::new(
            QueuedOperationType::Copy,
            params,
            OperationPriority::High,
        );

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: QueuedOperation = serde_json::from_str(&json).unwrap();

        assert_eq!(op.id, deserialized.id);
        assert_eq!(op.priority, deserialized.priority);
    }
}
