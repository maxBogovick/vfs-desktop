/**
 * System Operations Service
 *
 * System-level operations and information.
 */

use std::sync::Mutex;
use super::{ApiResult, ApiError};
use super::models::{FileSystemEntry, SystemStats, DirectorySize};
use crate::api::{RealFileSystem, virtual_fs::VirtualFileSystem, temporary_fs::TemporaryFileSystem};
use crate::config::FileSystemBackend;
use crate::core::FileSystem;
use crate::state::APP_CONFIG;
use crate::api_service::files::TEMP_FS_SESSIONS;
use sysinfo::{System, Pid, ProcessesToUpdate};

/// Enum для хранения разных типов файловых систем
enum FileSystemInstance {
    Real(RealFileSystem),
    Virtual(VirtualFileSystem),
    Temporary(TemporaryFileSystem),
}

impl FileSystemInstance {
    fn as_trait(&self) -> &dyn FileSystem {
        match self {
            FileSystemInstance::Real(fs) => fs,
            FileSystemInstance::Virtual(fs) => fs,
            FileSystemInstance::Temporary(fs) => fs,
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
    /// * `backend` - Optional backend type ("real" or "virtual" or a window label). If None, uses global config.
    fn get_filesystem_by_backend(&self, backend: Option<&str>) -> FileSystemInstance {
        // Special case: check if backend is a window label in VAULT_FS_SESSIONS
        if let Some(label) = backend {
            let sessions = crate::api_service::vault::VAULT_FS_SESSIONS.lock().unwrap();
            if let Some(vfs) = sessions.get(label) {
                tracing::debug!("Using window-specific VFS session for label: {}", label);
                return FileSystemInstance::Virtual(vfs.clone());
            }
        }

        // Check if backend is a temporary session
        if let Some(label) = backend {
            let sessions = TEMP_FS_SESSIONS.lock().unwrap();
            if let Some(tfs) = sessions.get(label) {
                tracing::debug!("Using temporary FS session for label: {}", label);
                return FileSystemInstance::Temporary(tfs.clone());
            }
        }

        let backend_enum = match backend {
            Some("real") => FileSystemBackend::Real,
            Some("virtual") => FileSystemBackend::Virtual,
            None => {
                // Fallback на глобальную конфигурацию
                let config = APP_CONFIG.read().unwrap();
                config.filesystem_backend.clone()
            }
            Some(_other) => {
                // Check if it was intended to be virtual but used a label not found yet
                // Fallback to global virtual if configured
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
                
                // Use shared VAULT_FS singleton to ensure we share the unlocked state
                use crate::api_service::vault::VAULT_FS;
                let mut vfs_guard = VAULT_FS.lock().unwrap();
                
                if vfs_guard.is_none() {
                    let virtual_fs = VirtualFileSystem::new_with_config()
                        .unwrap_or_else(|e| {
                            tracing::error!("Failed to initialize VirtualFileSystem: {}", e.message);
                            panic!("Cannot initialize VirtualFileSystem: {}", e.message);
                        });
                    *vfs_guard = Some(virtual_fs);
                }
                
                let vfs = vfs_guard.as_ref().unwrap().clone();
                FileSystemInstance::Virtual(vfs)
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

    /// Execute shell command with timeout
    pub fn execute_shell_command(&self, command: &str, working_dir: &str) -> ApiResult<super::models::CommandResult> {
        use std::process::{Command, Stdio};
        use std::time::Duration;
        use wait_timeout::ChildExt;

        // Spawn the process
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ApiError::System(format!("Failed to start command: {}", e)))?;

        // Wait with 30 second timeout
        let timeout = Duration::from_secs(30);
        match child.wait_timeout(timeout)
            .map_err(|e| ApiError::System(format!("Failed to wait for command: {}", e)))? {
            Some(_status) => {
                // Process finished within timeout, get output
                let output = child.wait_with_output()
                    .map_err(|e| ApiError::System(format!("Failed to get command output: {}", e)))?;

                Ok(super::models::CommandResult {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code().unwrap_or(-1),
                    success: output.status.success(),
                })
            }
            None => {
                // Timeout expired, kill the process
                let _ = child.kill();
                let _ = child.wait(); // Reap the zombie process

                Ok(super::models::CommandResult {
                    stdout: String::new(),
                    stderr: "Command timed out after 30 seconds. Long-running commands are not supported.".to_string(),
                    exit_code: -1,
                    success: false,
                })
            }
        }
    }
}

impl Default for SystemService {
    fn default() -> Self {
        Self::new()
    }
}