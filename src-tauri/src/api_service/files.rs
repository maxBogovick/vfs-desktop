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
use serde::{Deserialize, Serialize};
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

/// Результат batch создания файлов
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateResult {
    pub created: Vec<String>,
    pub failed: Vec<BatchCreateError>,
}

/// Ошибка при создании файла в batch операции
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateError {
    pub name: String,
    pub error: String,
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
        self.get_filesystem_by_backend(None)
    }

    /// Get filesystem instance based on optional backend parameter
    ///
    /// # Arguments
    /// * `backend` - Optional backend type ("real" or "virtual"). If None, uses global config.
    fn get_filesystem_by_backend(&self, backend: Option<&str>) -> FileSystemInstance {
        let backend_enum = match backend {
            Some("real") => FileSystemBackend::Real,
            Some("virtual") => FileSystemBackend::Virtual,
            None => {
                // Fallback на глобальную конфигурацию
                let config = APP_CONFIG.read().unwrap();
                config.filesystem_backend.clone()
            }
            Some(other) => {
                tracing::warn!("Invalid filesystem backend '{}', using global config", other);
                let config = APP_CONFIG.read().unwrap();
                config.filesystem_backend.clone()
            }
        };

        match backend_enum {
            FileSystemBackend::Real => {
                tracing::debug!("Using RealFileSystem backend");
                FileSystemInstance::Real(RealFileSystem::new())
            }
            FileSystemBackend::Virtual => {
                tracing::debug!("Using VirtualFileSystem backend");
                let virtual_fs = VirtualFileSystem::new_with_config()
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to initialize VirtualFileSystem: {}", e.message);
                        panic!("Cannot initialize VirtualFileSystem: {}", e.message);
                    });
                FileSystemInstance::Virtual(virtual_fs)
            }
        }
    }

    /// List directory contents
    ///
    /// # Arguments
    /// * `path` - Directory path to list
    /// * `panel_fs` - Optional filesystem backend ("real" or "virtual")
    ///
    /// # Returns
    /// Vector of FileSystemEntry or error if directory cannot be read
    pub fn list_directory(&self, path: &str, panel_fs: Option<&str>) -> ApiResult<Vec<FileSystemEntry>> {
        tracing::debug!("Listing directory: {} with backend: {:?}", path, panel_fs);

        self.get_filesystem_by_backend(panel_fs)
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
    pub fn get_file_info(&self, path: &str, panel_fs: Option<&str>) -> ApiResult<FileSystemEntry> {
        tracing::debug!("Getting file info: {} with backend: {:?}", path, panel_fs);

        self.get_filesystem_by_backend(panel_fs).as_trait().get_file_info(path).map_err(|err| {
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
    /// * `panel_fs` - Optional filesystem backend ("real" or "virtual")
    pub fn create_folder(&self, path: &str, name: &str, panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!("Creating folder '{}' in '{}' with backend: {:?}", name, path, panel_fs);

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

        self.get_filesystem_by_backend(panel_fs).as_trait().create_folder(path, name).map_err(|err| {
            tracing::error!("Failed to create folder: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Create new file with optional content
    ///
    /// # Arguments
    /// * `path` - Parent directory path
    /// * `name` - New file name
    /// * `content` - Optional file content
    /// * `panel_fs` - Optional filesystem backend ("real" or "virtual")
    pub fn create_file(&self, path: &str, name: &str, content: Option<&str>, panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!("Creating file '{}' in '{}' with backend: {:?}", name, path, panel_fs);

        // Validate file name
        if name.is_empty() {
            return Err(ApiError::ValidationError {
                message: "File name cannot be empty".to_string(),
            });
        }

        if name.contains('/') || name.contains('\\') {
            return Err(ApiError::ValidationError {
                message: "File name cannot contain path separators".to_string(),
            });
        }

        self.get_filesystem_by_backend(panel_fs).as_trait().create_file(path, name, content).map_err(|err| {
            tracing::error!("Failed to create file: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Batch create multiple files
    ///
    /// # Arguments
    /// * `path` - Parent directory path
    /// * `files` - Vector of (name, content) tuples
    /// * `panel_fs` - Optional filesystem backend ("real" or "virtual")
    pub fn create_files_batch(
        &self,
        path: &str,
        files: &[(String, Option<String>)],
        panel_fs: Option<&str>,
    ) -> ApiResult<BatchCreateResult> {
        tracing::info!("Batch creating {} files in '{}' with backend: {:?}", files.len(), path, panel_fs);

        if files.is_empty() {
            return Err(ApiError::ValidationError {
                message: "No files specified".to_string(),
            });
        }

        let results = self.get_filesystem_by_backend(panel_fs)
            .as_trait()
            .create_files_batch(path, files)
            .map_err(|err| {
                tracing::error!("Batch create failed: {}", err.message);
                ApiError::OperationFailed {
                    message: err.message,
                }
            })?;

        let mut created = Vec::new();
        let mut failed = Vec::new();

        for (i, result) in results.iter().enumerate() {
            match result {
                Ok(_) => created.push(files[i].0.clone()),
                Err(err) => failed.push(BatchCreateError {
                    name: files[i].0.clone(),
                    error: err.message.clone(),
                }),
            }
        }

        Ok(BatchCreateResult { created, failed })
    }

    /// Copy files/folders to destination
    /// `TODO change signature instead of panel_fs need using source_file_system, destination_file_system
    pub fn copy_items(&self, sources: &[String], destination: &str, panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!(
            "Copying {} items to '{}' with backend: {:?}",
            sources.len(),
            destination,
            panel_fs
        );

        if sources.is_empty() {
            return Err(ApiError::ValidationError {
                message: "No source files specified".to_string(),
            });
        }

        self.get_filesystem_by_backend(panel_fs).as_trait().copy_items(sources, destination).map_err(|err| {
            tracing::error!("Copy operation failed: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Move files/folders to destination
    pub fn move_items(&self, sources: &[String], destination: &str, panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!(
            "Moving {} items to '{}' with backend: {:?}",
            sources.len(),
            destination,
            panel_fs
        );

        if sources.is_empty() {
            return Err(ApiError::ValidationError {
                message: "No source files specified".to_string(),
            });
        }

        self.get_filesystem_by_backend(panel_fs).as_trait().move_items(sources, destination).map_err(|err| {
            tracing::error!("Move operation failed: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Rename file or folder
    pub fn rename_item(&self, old_path: &str, new_name: &str, panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!("Renaming '{}' to '{}' with backend: {:?}", old_path, new_name, panel_fs);

        if new_name.is_empty() {
            return Err(ApiError::ValidationError {
                message: "New name cannot be empty".to_string(),
            });
        }

        self.get_filesystem_by_backend(panel_fs).as_trait().rename_item(old_path, new_name).map_err(|err| {
            tracing::error!("Rename failed: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Delete single file or folder
    pub fn delete_item(&self, path: &str, panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!("Deleting '{}' with backend: {:?}", path, panel_fs);

        self.get_filesystem_by_backend(panel_fs).as_trait().delete_item(path).map_err(|err| {
            tracing::error!("Delete failed: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Delete multiple files/folders
    ///
    /// Stops on first error encountered
    pub fn delete_items(&self, paths: &[String], panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!("Deleting {} items with backend: {:?}", paths.len(), panel_fs);

        if paths.is_empty() {
            return Err(ApiError::ValidationError {
                message: "No files specified for deletion".to_string(),
            });
        }

        for path in paths {
            self.delete_item(path, panel_fs)?;
        }

        Ok(())
    }

    /// Read file content as string
    ///
    /// # Arguments
    /// * `path` - File path to read
    /// * `max_size` - Optional maximum size in bytes
    /// * `panel_fs` - Optional filesystem backend ("real" or "virtual")
    pub fn read_file_content(&self, path: &str, max_size: Option<u64>, panel_fs: Option<&str>) -> ApiResult<String> {
        tracing::debug!("Reading file content: {} with backend: {:?}", path, panel_fs);

        self.get_filesystem_by_backend(panel_fs).as_trait().read_file_content(path, max_size).map_err(|err| {
            tracing::error!("Failed to read file: {}", err.message);
            ApiError::FileNotFound {
                path: path.to_string(),
            }
        })
    }

    /// Write file content
    ///
    /// # Arguments
    /// * `path` - File path to write
    /// * `content` - Content to write to the file
    /// * `panel_fs` - Optional filesystem backend ("real" or "virtual")
    pub fn write_file_content(&self, path: &str, content: &str, panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!("Writing file content: {} with backend: {:?}", path, panel_fs);

        // Validate path
        if path.is_empty() {
            return Err(ApiError::ValidationError {
                message: "File path cannot be empty".to_string(),
            });
        }

        self.get_filesystem_by_backend(panel_fs).as_trait().write_file_content(path, content).map_err(|err| {
            tracing::error!("Failed to write file: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Open file with system default application
    pub fn open_file(&self, path: &str, panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!("Opening file: {} with backend: {:?}", path, panel_fs);

        self.get_filesystem_by_backend(panel_fs).as_trait().open_file(path).map_err(|err| {
            tracing::error!("Failed to open file: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Reveal file in system file manager
    pub fn reveal_in_finder(&self, path: &str, panel_fs: Option<&str>) -> ApiResult<()> {
        tracing::info!("Revealing in finder: {} with backend: {:?}", path, panel_fs);

        self.get_filesystem_by_backend(panel_fs).as_trait().reveal_in_finder(path).map_err(|err| {
            tracing::error!("Failed to reveal: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Normalize path (resolve .. and .)
    pub fn normalize_path(&self, path: &str, panel_fs: Option<&str>) -> ApiResult<String> {
        self.get_filesystem_by_backend(panel_fs).as_trait().normalize_path(path).map_err(|err| {
            ApiError::InvalidPath {
                path: path.to_string(),
            }
        })
    }

    /// Get path suggestions for autocomplete
    pub fn get_path_suggestions(&self, partial_path: &str, panel_fs: Option<&str>) -> ApiResult<Vec<String>> {
        self.get_filesystem_by_backend(panel_fs).as_trait()
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
    /// * `panel_fs` - Optional filesystem backend ("real" or "virtual")
    pub fn copy_with_custom_name(
        &self,
        source_path: &str,
        destination_dir: &str,
        new_name: &str,
        panel_fs: Option<&str>,
    ) -> ApiResult<()> {
        tracing::info!(
            "Copying '{}' to '{}' with custom name '{}' with backend: {:?}",
            source_path,
            destination_dir,
            new_name,
            panel_fs
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

        self.get_filesystem_by_backend(panel_fs).as_trait()
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
