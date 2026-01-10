// Queue module for managing batch operations
//
// This module provides a professional operations queue system with:
// - Parallel execution with configurable limits
// - Persistence (save/restore on app restart)
// - Scheduled tasks
// - Automatic retry with exponential backoff
// - Priority-based execution

pub mod types;
pub mod persistence;
pub mod scheduler;
pub mod executor;
pub mod manager;

// Re-export main types for convenience
pub use types::{
    QueuedOperation, QueuedOperationType, QueueOperationStatus, OperationPriority,
    OperationParams, RetryPolicy, RetryAttempt, QueueConfig,
};
pub use manager::{QueueManager, QUEUE_MANAGER};
