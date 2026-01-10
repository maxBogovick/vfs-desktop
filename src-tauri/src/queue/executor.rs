use super::types::*;
use crate::progress::{OPERATIONS_MANAGER, OperationType as CoreOperationType};
use crate::{file_operations, archives};
use tracing::{info, error, debug};

/// Operation executor that integrates with existing file operations
pub struct OperationExecutor;

impl OperationExecutor {
    /// Create a new executor
    pub fn new() -> Self {
        Self
    }

    /// Execute an operation
    ///
    /// This method dispatches to the appropriate handler based on operation type
    /// and integrates with the existing ProgressTracker system.
    pub fn execute(&self, operation: &QueuedOperation) -> Result<(), String> {
        info!("Executing operation: {} ({:?})", operation.id, operation.operation_type);
        debug!("Operation params: {:?}", operation.params);

        let result = match &operation.params {
            OperationParams::Copy { sources, destination, source_fs, dest_fs } => {
                self.execute_copy(operation, sources, destination, source_fs, dest_fs)
            }
            OperationParams::Move { sources, destination, source_fs, dest_fs } => {
                self.execute_move(operation, sources, destination, source_fs, dest_fs)
            }
            OperationParams::Delete { paths, panel_fs, source_fs } => {
                // Prefer source_fs, fall back to panel_fs
                let fs = source_fs.as_ref().or(panel_fs.as_ref()).map(|s| s.clone());
                self.execute_delete(operation, paths, fs)
            }
            OperationParams::Archive { sources, archive_path, format, source_fs, dest_fs } => {
                self.execute_archive(operation, sources, archive_path, format, source_fs, dest_fs)
            }
            OperationParams::Extract { archive_path, destination, source_fs, dest_fs } => {
                self.execute_extract(operation, archive_path, destination, source_fs, dest_fs)
            }
            OperationParams::BatchRename { items, config, source_fs } => {
                self.execute_batch_rename(operation, items, config, source_fs)
            }
            OperationParams::BatchAttribute { items, config, source_fs } => {
                self.execute_batch_attribute(operation, items, config, source_fs)
            }
            OperationParams::Custom { command, args } => {
                self.execute_custom(operation, command, args)
            }
        };

        if let Err(e) = &result {
            error!("Operation {} failed: {}", operation.id, e);
        }

        result
    }

    /// Execute copy operation
    fn execute_copy(
        &self,
        operation: &QueuedOperation,
        sources: &[String],
        destination: &str,
        source_fs: &Option<String>,
        dest_fs: &Option<String>,
    ) -> Result<(), String> {
        info!("=== COPY OPERATION ===");
        info!("Operation ID: {}", operation.id);
        info!("Sources ({} items): {:?}", sources.len(), sources);
        info!("Destination: {}", destination);
        info!("Source FS: {:?}", source_fs);
        info!("Dest FS: {:?}", dest_fs);

        // Calculate total size
        info!("Calculating total size...");
        let (total_bytes, total_items) = file_operations::calculate_total_size(sources)
            .map_err(|e| {
                let err_msg = format!("Failed to calculate size: {}", e);
                error!("{}", err_msg);
                err_msg
            })?;
        info!("Total size: {} bytes, {} items", total_bytes, total_items);

        // Create ProgressTracker for integration
        let tracker = OPERATIONS_MANAGER.create_operation(
            operation.id.clone(),
            CoreOperationType::Copy,
            total_bytes,
            total_items,
        );

        // Note: For queue operations, we don't have AppHandle here
        // The manager will need to provide it or we'll use a different approach
        // For now, we'll call the function without emit (it will work but won't emit events)

        // Execute copy operation
        info!("Starting file copy...");
        let copy_result = file_operations::copy_items_simple(
            sources,
            destination,
            &tracker,
            source_fs.as_deref(),
            dest_fs.as_deref(),
        );

        match copy_result {
            Ok(_) => {
                tracker.mark_completed();
                info!("✓ Copy operation completed successfully: {}", operation.id);
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Copy failed: {}", e.message);
                error!("✗ {}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Execute move operation
    fn execute_move(
        &self,
        operation: &QueuedOperation,
        sources: &[String],
        destination: &str,
        source_fs: &Option<String>,
        dest_fs: &Option<String>,
    ) -> Result<(), String> {
        info!("=== MOVE OPERATION ===");
        info!("Operation ID: {}", operation.id);
        info!("Sources: {:?}", sources);
        info!("Destination: {}", destination);

        let (total_bytes, total_items) = file_operations::calculate_total_size(sources)
            .map_err(|e| format!("Failed to calculate size: {}", e))?;

        let tracker = OPERATIONS_MANAGER.create_operation(
            operation.id.clone(),
            CoreOperationType::Move,
            total_bytes,
            total_items,
        );

        let result = file_operations::move_items_simple(
            sources,
            destination,
            &tracker,
            source_fs.as_deref(),
            dest_fs.as_deref(),
        );

        match result {
            Ok(_) => {
                tracker.mark_completed();
                info!("✓ Move operation completed successfully");
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Move failed: {}", e.message);
                error!("✗ {}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Execute delete operation
    fn execute_delete(
        &self,
        operation: &QueuedOperation,
        paths: &[String],
        fs: Option<String>,
    ) -> Result<(), String> {
        info!("=== DELETE OPERATION ===");
        info!("Operation ID: {}", operation.id);
        info!("Paths: {:?}", paths);
        info!("FS: {:?}", fs);

        let (total_bytes, total_items) = file_operations::calculate_total_size(paths)
            .map_err(|e| format!("Failed to calculate size: {}", e))?;

        let tracker = OPERATIONS_MANAGER.create_operation(
            operation.id.clone(),
            CoreOperationType::Delete,
            total_bytes,
            total_items,
        );

        let result = file_operations::delete_items_simple(
            paths,
            &tracker,
            fs.as_deref(),
        );

        match result {
            Ok(_) => {
                tracker.mark_completed();
                info!("✓ Delete operation completed successfully");
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Delete failed: {}", e.message);
                error!("✗ {}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Execute archive creation
    fn execute_archive(
        &self,
        operation: &QueuedOperation,
        sources: &[String],
        archive_path: &str,
        _format: &Option<String>,
        source_fs: &Option<String>,
        dest_fs: &Option<String>,
    ) -> Result<(), String> {
        info!("=== ARCHIVE OPERATION ===");
        info!("Operation ID: {}", operation.id);
        info!("Sources: {:?}", sources);
        info!("Archive path: {}", archive_path);
        info!("Source FS: {:?}", source_fs);
        info!("Dest FS: {:?}", dest_fs);

        let result = archives::create_archive_with_fs(
            sources.to_vec(),
            archive_path.to_string(),
            source_fs.as_deref(),
            dest_fs.as_deref()
        );

        match result {
            Ok(_) => {
                info!("✓ Archive created successfully");
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Archive creation failed: {}", e);
                error!("✗ {}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Execute archive extraction
    fn execute_extract(
        &self,
        operation: &QueuedOperation,
        archive_path: &str,
        destination: &str,
        source_fs: &Option<String>,
        dest_fs: &Option<String>,
    ) -> Result<(), String> {
        info!("=== EXTRACT OPERATION ===");
        info!("Operation ID: {}", operation.id);
        info!("Archive path: {}", archive_path);
        info!("Destination: {}", destination);
        info!("Source FS: {:?}", source_fs);
        info!("Dest FS: {:?}", dest_fs);

        let result = archives::extract_archive_with_fs(
            archive_path,
            destination,
            source_fs.as_deref(),
            dest_fs.as_deref()
        );

        match result {
            Ok(_) => {
                info!("✓ Archive extracted successfully");
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Extraction failed: {}", e);
                error!("✗ {}", err_msg);
                Err(err_msg)
            }
        }
    }

    /// Execute batch rename operation
    fn execute_batch_rename(
        &self,
        operation: &QueuedOperation,
        items: &[String],
        config: &serde_json::Value,
        source_fs: &Option<String>,
    ) -> Result<(), String> {
        info!("Batch renaming {} items", items.len());
        info!("Source FS: {:?}", source_fs);

        // TODO: Implement batch rename logic
        // This will need to:
        // 1. Parse config from frontend
        // 2. Apply renaming patterns
        // 3. Handle conflicts
        // 4. Track progress

        error!("Batch rename not yet implemented for queue");
        Err("Batch rename not implemented yet".to_string())
    }

    /// Execute batch attribute change operation
    fn execute_batch_attribute(
        &self,
        operation: &QueuedOperation,
        items: &[String],
        config: &serde_json::Value,
        source_fs: &Option<String>,
    ) -> Result<(), String> {
        info!("Batch changing attributes for {} items", items.len());
        info!("Source FS: {:?}", source_fs);

        // TODO: Implement batch attribute change logic
        // This will need to:
        // 1. Parse config from frontend
        // 2. Apply attribute changes (permissions, dates, etc.)
        // 3. Track progress

        error!("Batch attribute change not yet implemented for queue");
        Err("Batch attribute change not implemented yet".to_string())
    }

    /// Execute custom operation
    fn execute_custom(
        &self,
        operation: &QueuedOperation,
        command: &str,
        args: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        info!("Executing custom operation: {}", command);

        // TODO: Implement custom operation registry
        // This will allow registering custom handlers

        error!("Custom operations not yet implemented");
        Err("Custom operations not implemented yet".to_string())
    }
}

impl Default for OperationExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = OperationExecutor::new();
        // Just verify it can be created
        assert!(true);
    }
}
