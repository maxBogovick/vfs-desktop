use chrono::{DateTime, Utc};

/// Task scheduler for managing scheduled operations
pub struct TaskScheduler;

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new() -> Self {
        Self
    }

    /// Check if it's time to execute a scheduled task
    pub fn is_time_to_execute(&self, scheduled_at: DateTime<Utc>) -> bool {
        Utc::now() >= scheduled_at
    }

    /// Get time remaining until execution (in seconds)
    pub fn time_until_execution(&self, scheduled_at: DateTime<Utc>) -> i64 {
        let now = Utc::now();
        if now >= scheduled_at {
            return 0;
        }
        (scheduled_at - now).num_seconds()
    }

    /// Check if a task is overdue
    pub fn is_overdue(&self, scheduled_at: DateTime<Utc>) -> bool {
        let now = Utc::now();
        now > scheduled_at
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_is_time_to_execute() {
        let scheduler = TaskScheduler::new();

        // Past time should be ready
        let past = Utc::now() - Duration::hours(1);
        assert!(scheduler.is_time_to_execute(past));

        // Future time should not be ready
        let future = Utc::now() + Duration::hours(1);
        assert!(!scheduler.is_time_to_execute(future));

        // Current time should be ready
        let now = Utc::now();
        assert!(scheduler.is_time_to_execute(now));
    }

    #[test]
    fn test_time_until_execution() {
        let scheduler = TaskScheduler::new();

        // Future time
        let future = Utc::now() + Duration::hours(1);
        let remaining = scheduler.time_until_execution(future);
        assert!(remaining > 3500 && remaining <= 3600); // Around 1 hour

        // Past time
        let past = Utc::now() - Duration::hours(1);
        assert_eq!(scheduler.time_until_execution(past), 0);
    }

    #[test]
    fn test_is_overdue() {
        let scheduler = TaskScheduler::new();

        // Past time is overdue
        let past = Utc::now() - Duration::hours(1);
        assert!(scheduler.is_overdue(past));

        // Future time is not overdue
        let future = Utc::now() + Duration::hours(1);
        assert!(!scheduler.is_overdue(future));
    }
}
