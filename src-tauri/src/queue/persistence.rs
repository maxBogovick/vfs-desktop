use super::types::QueuedOperation;
use std::fs;
use std::path::PathBuf;
use serde_json;
use tracing::{debug, warn, error};

/// Handles persistence of the operations queue to disk
pub struct QueuePersistence {
    file_path: PathBuf,
}

impl QueuePersistence {
    /// Create a new persistence handler
    pub fn new() -> Self {
        let file_path = Self::get_persistence_path();
        debug!("Queue persistence path: {}", file_path.display());
        Self { file_path }
    }

    /// Get the path where the queue is persisted
    fn get_persistence_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        let vfdir_config = config_dir.join("vfdir");
        vfdir_config.join("operations_queue.json")
    }

    /// Save operations to disk
    pub fn save(&self, operations: &[QueuedOperation]) -> Result<(), String> {
        // Create config directory if it doesn't exist
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // Serialize to JSON with pretty printing
        let json = serde_json::to_string_pretty(operations)
            .map_err(|e| format!("Failed to serialize operations: {}", e))?;

        // Write to temporary file first, then rename (atomic operation)
        let temp_path = self.file_path.with_extension("tmp");
        fs::write(&temp_path, &json)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        fs::rename(&temp_path, &self.file_path)
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;

        debug!("Saved {} operations to disk", operations.len());
        Ok(())
    }

    /// Load operations from disk
    pub fn load(&self) -> Result<Vec<QueuedOperation>, String> {
        if !self.file_path.exists() {
            debug!("Queue file does not exist, starting with empty queue");
            return Ok(Vec::new());
        }

        let json = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read queue file: {}", e))?;

        let operations: Vec<QueuedOperation> = serde_json::from_str(&json)
            .map_err(|e| {
                error!("Failed to deserialize queue: {}", e);
                format!("Failed to deserialize operations: {}", e)
            })?;

        debug!("Loaded {} operations from disk", operations.len());
        Ok(operations)
    }

    /// Delete the persisted queue file
    pub fn clear(&self) -> Result<(), String> {
        if self.file_path.exists() {
            fs::remove_file(&self.file_path)
                .map_err(|e| format!("Failed to delete queue file: {}", e))?;
            debug!("Cleared queue file");
        }
        Ok(())
    }

    /// Check if a persisted queue exists
    pub fn exists(&self) -> bool {
        self.file_path.exists()
    }

    /// Get the size of the persisted queue file
    pub fn file_size(&self) -> Option<u64> {
        fs::metadata(&self.file_path)
            .ok()
            .map(|m| m.len())
    }
}

impl Default for QueuePersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::types::{QueuedOperationType, OperationPriority, OperationParams};
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().join("test_queue.json");

        let mut persistence = QueuePersistence::new();
        persistence.file_path = test_path.clone();

        // Create test operations
        let params = OperationParams::Copy {
            sources: vec!["/test/file1".to_string()],
            destination: "/test/dest".to_string(),
            source_fs: None,
            dest_fs: None,
        };

        let op1 = QueuedOperation::new(
            QueuedOperationType::Copy,
            params.clone(),
            OperationPriority::Normal,
        );

        let op2 = QueuedOperation::new(
            QueuedOperationType::Move,
            params,
            OperationPriority::High,
        );

        let operations = vec![op1.clone(), op2.clone()];

        // Save
        persistence.save(&operations).unwrap();
        assert!(test_path.exists());

        // Load
        let loaded = persistence.load().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, op1.id);
        assert_eq!(loaded[1].id, op2.id);
        assert_eq!(loaded[0].operation_type, QueuedOperationType::Copy);
        assert_eq!(loaded[1].operation_type, QueuedOperationType::Move);
    }

    #[test]
    fn test_load_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().join("nonexistent.json");

        let mut persistence = QueuePersistence::new();
        persistence.file_path = test_path;

        let loaded = persistence.load().unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_clear() {
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().join("test_queue.json");

        let mut persistence = QueuePersistence::new();
        persistence.file_path = test_path.clone();

        // Save something
        persistence.save(&vec![]).unwrap();
        assert!(test_path.exists());

        // Clear
        persistence.clear().unwrap();
        assert!(!test_path.exists());
    }
}
