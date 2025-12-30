/**
 * File Operations Service
 *
 * Comprehensive file system operations with proper error handling.
 *
 * # Examples
 *
 * ```rust
 * use vfdir::api_service::FileService;
 *
 * let service = FileService::new();
 * let files = service.list_directory("/Users")?;
 * ```
 */

use super::{ApiResult, ApiError};
use super::models::FileSystemEntry;
use crate::api::{RealFileSystem, virtual_fs::VirtualFileSystem};
use crate::config::FileSystemBackend;
use crate::core::FileSystem;
use crate::state::APP_CONFIG;
use std::path::Path;

/// Enum для хранения разных типов файловых систем
enum FileSystemInstance {
    Real(RealFileSystem),
    Virtual(VirtualFileSystem),
}

impl FileSystemInstance {
    fn as_trait(&self) -> &dyn FileSystem {
        match self {
            FileSystemInstance::Real(fs) => fs,
            FileSystemInstance::Virtual(fs) => fs,
        }
    }
}

/// Service for file system operations
///
/// Thread-safe and can be shared across async contexts
pub struct FileService;

impl FileService {
    /// Create a new file service instance
    pub fn new() -> Self {
        tracing::debug!("Initializing FileService");
        Self
    }

    /// Get filesystem instance based on configuration
    fn get_filesystem(&self) -> FileSystemInstance {
        let config = APP_CONFIG.read().unwrap();

        match config.filesystem_backend {
            FileSystemBackend::Real => {
                tracing::debug!("Using RealFileSystem backend");
                FileSystemInstance::Real(RealFileSystem::new())
            }
            FileSystemBackend::Virtual => {
                tracing::debug!("Using VirtualFileSystem backend");
                let virtual_fs = VirtualFileSystem::new("/Users/maxim/Projects/Rust/vfdir/out/fs.json")
                    .unwrap_or_else(|_| VirtualFileSystem::new("/tmp/vfdir_fs.json").unwrap());
                FileSystemInstance::Virtual(virtual_fs)
            }
        }
    }

    /// List directory contents
    ///
    /// # Arguments
    /// * `path` - Directory path to list
    ///
    /// # Returns
    /// Vector of FileSystemEntry or error if directory cannot be read
    pub fn list_directory(&self, path: &str) -> ApiResult<Vec<FileSystemEntry>> {
        tracing::debug!("Listing directory: {}", path);

        self.get_filesystem()
            .as_trait()
            .read_directory(path)
            .map_err(|err| {
                tracing::error!("Failed to list directory '{}': {}", path, err.message);
                ApiError::OperationFailed {
                    message: err.message,
                }
            })
    }

    /// Get file or directory information
    pub fn get_file_info(&self, path: &str) -> ApiResult<FileSystemEntry> {
        tracing::debug!("Getting file info: {}", path);

        self.get_filesystem().as_trait().get_file_info(path).map_err(|err| {
            tracing::warn!("File not found: {}", path);
            ApiError::FileNotFound {
                path: path.to_string(),
            }
        })
    }

    /// Create new folder
    ///
    /// # Arguments
    /// * `path` - Parent directory path
    /// * `name` - New folder name
    pub fn create_folder(&self, path: &str, name: &str) -> ApiResult<()> {
        tracing::info!("Creating folder '{}' in '{}'", name, path);

        // Validate folder name
        if name.is_empty() {
            return Err(ApiError::ValidationError {
                message: "Folder name cannot be empty".to_string(),
            });
        }

        if name.contains('/') || name.contains('\\') {
            return Err(ApiError::ValidationError {
                message: "Folder name cannot contain path separators".to_string(),
            });
        }

        self.get_filesystem().as_trait().create_folder(path, name).map_err(|err| {
            tracing::error!("Failed to create folder: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Copy files/folders to destination
    pub fn copy_items(&self, sources: &[String], destination: &str) -> ApiResult<()> {
        tracing::info!(
            "Copying {} items to '{}'",
            sources.len(),
            destination
        );

        if sources.is_empty() {
            return Err(ApiError::ValidationError {
                message: "No source files specified".to_string(),
            });
        }

        self.get_filesystem().as_trait().copy_items(sources, destination).map_err(|err| {
            tracing::error!("Copy operation failed: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Move files/folders to destination
    pub fn move_items(&self, sources: &[String], destination: &str) -> ApiResult<()> {
        tracing::info!(
            "Moving {} items to '{}'",
            sources.len(),
            destination
        );

        if sources.is_empty() {
            return Err(ApiError::ValidationError {
                message: "No source files specified".to_string(),
            });
        }

        self.get_filesystem().as_trait().move_items(sources, destination).map_err(|err| {
            tracing::error!("Move operation failed: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Rename file or folder
    pub fn rename_item(&self, old_path: &str, new_name: &str) -> ApiResult<()> {
        tracing::info!("Renaming '{}' to '{}'", old_path, new_name);

        if new_name.is_empty() {
            return Err(ApiError::ValidationError {
                message: "New name cannot be empty".to_string(),
            });
        }

        self.get_filesystem().as_trait().rename_item(old_path, new_name).map_err(|err| {
            tracing::error!("Rename failed: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Delete single file or folder
    pub fn delete_item(&self, path: &str) -> ApiResult<()> {
        tracing::info!("Deleting '{}'", path);

        self.get_filesystem().as_trait().delete_item(path).map_err(|err| {
            tracing::error!("Delete failed: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Delete multiple files/folders
    ///
    /// Stops on first error encountered
    pub fn delete_items(&self, paths: &[String]) -> ApiResult<()> {
        tracing::info!("Deleting {} items", paths.len());

        if paths.is_empty() {
            return Err(ApiError::ValidationError {
                message: "No files specified for deletion".to_string(),
            });
        }

        for path in paths {
            self.delete_item(path)?;
        }

        Ok(())
    }

    /// Read file content as string
    ///
    /// # Arguments
    /// * `path` - File path to read
    /// * `max_size` - Optional maximum size in bytes
    pub fn read_file_content(&self, path: &str, max_size: Option<u64>) -> ApiResult<String> {
        tracing::debug!("Reading file content: {}", path);

        self.get_filesystem().as_trait().read_file_content(path, max_size).map_err(|err| {
            tracing::error!("Failed to read file: {}", err.message);
            ApiError::FileNotFound {
                path: path.to_string(),
            }
        })
    }

    /// Open file with system default application
    pub fn open_file(&self, path: &str) -> ApiResult<()> {
        tracing::info!("Opening file: {}", path);

        self.get_filesystem().as_trait().open_file(path).map_err(|err| {
            tracing::error!("Failed to open file: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Reveal file in system file manager
    pub fn reveal_in_finder(&self, path: &str) -> ApiResult<()> {
        tracing::info!("Revealing in finder: {}", path);

        self.get_filesystem().as_trait().reveal_in_finder(path).map_err(|err| {
            tracing::error!("Failed to reveal: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Normalize path (resolve .. and .)
    pub fn normalize_path(&self, path: &str) -> ApiResult<String> {
        self.get_filesystem().as_trait().normalize_path(path).map_err(|err| {
            ApiError::InvalidPath {
                path: path.to_string(),
            }
        })
    }

    /// Get path suggestions for autocomplete
    pub fn get_path_suggestions(&self, partial_path: &str) -> ApiResult<Vec<String>> {
        self.get_filesystem().as_trait()
            .get_path_suggestions(partial_path)
            .map_err(|err| ApiError::OperationFailed {
                message: err.message,
            })
    }

    /// Copy file with custom name (for conflict resolution)
    ///
    /// # Arguments
    /// * `source_path` - Source file path
    /// * `destination_dir` - Destination directory
    /// * `new_name` - Custom name for the copied file
    pub fn copy_with_custom_name(
        &self,
        source_path: &str,
        destination_dir: &str,
        new_name: &str,
    ) -> ApiResult<()> {
        tracing::info!(
            "Copying '{}' to '{}' with custom name '{}'",
            source_path,
            destination_dir,
            new_name
        );

        // Validate inputs
        if new_name.is_empty() {
            return Err(ApiError::ValidationError {
                message: "File name cannot be empty".to_string(),
            });
        }

        if new_name.contains('/') || new_name.contains('\\') {
            return Err(ApiError::ValidationError {
                message: "File name cannot contain path separators".to_string(),
            });
        }

        self.get_filesystem().as_trait()
            .copy_with_custom_name(source_path, destination_dir, new_name)
            .map_err(|err| {
                tracing::error!("Failed to copy with custom name: {}", err.message);
                ApiError::OperationFailed {
                    message: err.message,
                }
            })
    }

    /// Check if file exists
    pub fn file_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    /// Get file extension
    pub fn get_extension(&self, path: &str) -> Option<String> {
        std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string())
    }
}

impl Default for FileService {
    fn default() -> Self {
        Self::new()
    }
}
