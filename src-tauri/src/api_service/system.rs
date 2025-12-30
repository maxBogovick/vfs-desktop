/**
 * System Operations Service
 *
 * System-level operations and information.
 */

use super::{ApiResult, ApiError};
use super::models::{FileSystemEntry, SystemStats};
use crate::api::{RealFileSystem, virtual_fs::VirtualFileSystem};
use crate::config::FileSystemBackend;
use crate::core::FileSystem;
use crate::state::APP_CONFIG;

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

pub struct SystemService;

impl SystemService {
    pub fn new() -> Self {
        tracing::debug!("Initializing SystemService");
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

    /// Get user's home directory path
    pub fn get_home_directory(&self) -> ApiResult<String> {
        tracing::debug!("Getting home directory");

        self.get_filesystem().as_trait().get_home_directory().map_err(|err| {
            tracing::error!("Failed to get home directory: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Get system folders (Desktop, Documents, Downloads, etc.)
    pub fn get_system_folders(&self) -> ApiResult<Vec<FileSystemEntry>> {
        tracing::debug!("Getting system folders");

        self.get_filesystem().as_trait().get_system_folders().map_err(|err| {
            tracing::error!("Failed to get system folders: {}", err.message);
            ApiError::OperationFailed {
                message: err.message,
            }
        })
    }

    /// Get current process system statistics
    pub fn get_stats(&self) -> ApiResult<SystemStats> {
        tracing::debug!("Getting system stats");
        use sysinfo::{System, Pid};

        let mut sys = System::new_all();
        let pid = Pid::from_u32(std::process::id());
        sys.refresh_all();

        if let Some(process) = sys.process(pid) {
            let stats = SystemStats {
                memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
                cpu_percent: process.cpu_usage(),
            };
            tracing::debug!("System stats: memory={:.2}MB, cpu={:.1}%", stats.memory_mb, stats.cpu_percent);
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
    pub fn calculate_directory_size(&self, _path: &str) -> ApiResult<u64> {
        // TODO: Implement proper directory size calculation
        tracing::warn!("Directory size calculation not yet implemented");
        Err(ApiError::OperationFailed {
            message: "Directory size calculation not yet implemented".to_string(),
        })
    }
}

impl Default for SystemService {
    fn default() -> Self {
        Self::new()
    }
}
