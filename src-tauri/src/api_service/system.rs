/**
 * System Operations Service
 *
 * System-level operations and information.
 */

use std::sync::Mutex;
use super::{ApiResult, ApiError};
use super::models::{FileSystemEntry, SystemStats, DirectorySize};
use crate::api::{RealFileSystem, virtual_fs::VirtualFileSystem};
use crate::config::FileSystemBackend;
use crate::core::FileSystem;
use crate::state::APP_CONFIG;
use sysinfo::{System, Pid, ProcessesToUpdate};

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

pub struct SystemService {
    sys: Mutex<System>,
}

impl SystemService {
    pub fn new() -> Self {
        tracing::debug!("Initializing SystemService");
        Self {
            sys: Mutex::new(System::new_all()),
        }
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

    /// Get user's home directory path
    pub fn get_home_directory(&self, panel_fs: Option<&str>) -> ApiResult<String> {
        tracing::debug!("Getting home directory with backend: {:?}", panel_fs);

        self.get_filesystem_by_backend(panel_fs).as_trait().get_home_directory().map_err(|err| {
            tracing::error!("Failed to get home directory: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Get system folders (Desktop, Documents, Downloads, etc.)
    pub fn get_system_folders(&self, panel_fs: Option<&str>) -> ApiResult<Vec<FileSystemEntry>> {
        tracing::debug!("Getting system folders with backend: {:?}", panel_fs);

        self.get_filesystem_by_backend(panel_fs).as_trait().get_system_folders().map_err(|err| {
            tracing::error!("Failed to get system folders: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Get current process system statistics
    pub fn get_stats(&self) -> ApiResult<SystemStats> {
        // tracing::debug!("Getting system stats"); // Reduced log spam
        
        let mut sys = self.sys.lock().map_err(|_| ApiError::Internal {
            message: "Failed to lock system info".to_string(),
        })?;
        
        let pid = Pid::from_u32(std::process::id());
        
        // Refresh only the specific process to get accurate CPU usage
        sys.refresh_processes(ProcessesToUpdate::All, true);

        if let Some(process) = sys.process(pid) {
            let stats = SystemStats {
                memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
                cpu_percent: process.cpu_usage(),
            };
            // tracing::debug!("System stats: memory={:.2}MB, cpu={:.1}%", stats.memory_mb, stats.cpu_percent);
            Ok(stats)
        } else {
            tracing::error!("Failed to get process information");
            Err(ApiError::Internal {
                message: "Failed to get process information".to_string(),
            })
        }
    }

    /// Open terminal at specified path
    pub fn open_terminal(&self, path: &str) -> ApiResult<()> {
        tracing::info!("Opening terminal at: {}", path);

        self.get_filesystem().as_trait().open_terminal(path).map_err(|err| {
            tracing::error!("Failed to open terminal: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Calculate total size of directory (recursive)
    pub fn calculate_directory_size(&self, path: &str) -> ApiResult<DirectorySize> {
        use crate::file_operations::calculate_total_size;

        tracing::debug!("Calculating directory size for: {}", path);

        let (total_bytes, total_items) = calculate_total_size(&[path.to_string()])
            .map_err(|e| {
                tracing::error!("Failed to calculate directory size: {}", e);
                ApiError::OperationFailed {
                    message: format!("Failed to calculate directory size: {}", e),
                }
            })?;

        Ok(DirectorySize {
            total_bytes,
            total_items,
        })
    }
}

impl Default for SystemService {
    fn default() -> Self {
        Self::new()
    }
}
