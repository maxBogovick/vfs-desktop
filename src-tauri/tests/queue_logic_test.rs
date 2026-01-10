use vfdir_lib::queue::{
    QUEUE_MANAGER, QueuedOperation, QueuedOperationType, OperationParams, 
    OperationPriority, QueueOperationStatus
};
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
