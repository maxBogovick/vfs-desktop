/**
 * Vault Security Service
 *
 * Manages encrypted virtual file system vault operations.
 */

use super::{ApiResult, ApiError};
use crate::api::{virtual_fs::VirtualFileSystem, temporary_fs::TemporaryFileSystem};
use crate::api::vault_error::{VaultError, VaultErrorResponse};
use crate::api::recovery::{RecoveryManager, RecoverySession};
use crate::api::notification_channels::ChannelConfig;
use crate::config::FileSystemBackend;
use crate::state::APP_CONFIG;
use crate::api_service::files::TEMP_FS_SESSIONS;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use once_cell::sync::Lazy;

use std::collections::HashMap;

/// Metadata for an active steganography session
struct StegoSession {
    container_path: PathBuf,
    password: String,
    temp_dir: PathBuf,
}

/// Map of active stego sessions
static STEGO_SESSIONS: Lazy<Arc<Mutex<HashMap<String, StegoSession>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// Map of window label to its specific VirtualFileSystem instance
pub type WindowFsMap = Arc<Mutex<HashMap<String, VirtualFileSystem>>>;

/// Global storage for all active VFS sessions
pub static VAULT_FS_SESSIONS: Lazy<WindowFsMap> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// Legacy global instance (for main window or when label not provided)
pub static VAULT_FS: Lazy<Arc<Mutex<Option<VirtualFileSystem>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

/// Active recovery session (temporary, single session)
static RECOVERY_SESSION: Lazy<Arc<Mutex<Option<RecoverySession>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

/// Check if Virtual File System backend is enabled
fn is_virtual_backend() -> bool {
    let config = APP_CONFIG.read().unwrap();
    config.filesystem_backend == FileSystemBackend::Virtual
}

/// Initialize or get the global VirtualFileSystem
fn get_or_init_vfs() -> Result<VirtualFileSystem, VaultError> {
    // Check if virtual backend is enabled - REMOVED to allow mixed mode
    // if !is_virtual_backend() {
    //     return Err(VaultError::CryptoError("Vault is only available for Virtual File System backend".into()));
    // }

    let mut vfs_guard = VAULT_FS.lock().unwrap();

    if vfs_guard.is_none() {
        let vfs = VirtualFileSystem::new_with_config()
            .map_err(|e| VaultError::CryptoError(format!("Failed to initialize VFS: {}", e.message)))?;
        *vfs_guard = Some(vfs);
    }

    // Clone the VirtualFileSystem (cheap because it uses Arc internally)
    Ok(vfs_guard.as_ref().unwrap().clone())
}

/// Vault operations service
pub struct VaultService;

impl VaultService {
    pub fn new() -> Self {
        tracing::debug!("Initializing VaultService");
        Self
    }

    /// Check if vault is enabled (only for Virtual FS backend)
    pub fn is_enabled(&self) -> ApiResult<bool> {
        Ok(is_virtual_backend())
    }

    /// Get current vault status
    pub fn get_status(&self) -> ApiResult<String> {
        // Return "DISABLED" if not using virtual backend - REMOVED to allow mixed mode
        // if !is_virtual_backend() {
        //     return Ok("DISABLED".to_string());
        // }

        let vfs = get_or_init_vfs()
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;
        Ok(vfs.get_vault_status())
    }

    /// Initialize vault with password (first-time setup)
    pub fn initialize(&self, password: String) -> ApiResult<()> {
        let vfs = get_or_init_vfs()
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        vfs.initialize_vault(&password)
            .map_err(|e| match e {
                VaultError::CryptoError(msg) => ApiError::Internal { message: msg },
                VaultError::Io(e) => ApiError::Internal { message: e.to_string() },
                _ => ApiError::Internal { message: e.to_string() },
            })
    }

    /// Unlock vault with password
    pub fn unlock(&self, password: String) -> ApiResult<()> {
        let vfs = get_or_init_vfs()
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        vfs.unlock_vault(&password)
            .map_err(|e| match e {
                VaultError::InvalidPassword => ApiError::ValidationError { message: "Invalid password".to_string() },
                VaultError::DecryptionFailed => ApiError::Internal { message: "Decryption failed".to_string() },
                VaultError::Locked => ApiError::ValidationError { message: "Vault is locked".to_string() },
                _ => ApiError::Internal { message: e.to_string() },
            })
    }

    /// Lock vault (clear keys from memory)
    pub fn lock(&self) -> ApiResult<()> {
        let vfs = get_or_init_vfs()
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        vfs.lock_vault()
            .map_err(|e| ApiError::Internal { message: e.to_string() })
    }

    // ==================== RECOVERY METHODS ====================

    /// Setup recovery channels for vault
    pub fn setup_recovery(&self, channels: Vec<ChannelConfig>) -> ApiResult<String> {
        let vfs = get_or_init_vfs()
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        vfs.setup_recovery(channels)
            .map_err(|e| ApiError::Internal { message: e.to_string() })
    }

    /// Request password reset (sends code to notification channel)
    pub fn request_password_reset(&self, channel_type: String) -> ApiResult<()> {
        let vfs = get_or_init_vfs()
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        vfs.request_password_reset(&channel_type)
            .map_err(|e| match e {
                VaultError::CryptoError(msg) => ApiError::ValidationError { message: msg },
                _ => ApiError::Internal { message: e.to_string() },
            })
    }

    /// Verify reset code and set new password
    pub fn verify_reset_code(&self, code: String, new_password: String) -> ApiResult<()> {
        let vfs = get_or_init_vfs()
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        vfs.complete_password_reset(&code, &new_password)
            .map_err(|e| match e {
                VaultError::InvalidPassword => ApiError::ValidationError { message: "Invalid verification code".to_string() },
                VaultError::CryptoError(msg) => ApiError::ValidationError { message: msg },
                _ => ApiError::Internal { message: e.to_string() },
            })
    }

    /// Get configured recovery channels
    pub fn get_recovery_channels(&self) -> ApiResult<Vec<String>> {
        let vfs = get_or_init_vfs()
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        vfs.get_recovery_channels()
            .map_err(|e| ApiError::Internal { message: e.to_string() })
    }

    /// Check if recovery is configured for the vault
    pub fn is_recovery_configured(&self) -> ApiResult<bool> {
        let vfs = get_or_init_vfs()
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        // Try to get recovery channels, if it succeeds then recovery is configured
        match vfs.get_recovery_channels() {
            Ok(channels) => {
                let is_configured = !channels.is_empty();
                println!("[Backend] Recovery configured: {}, channels: {:?}", is_configured, channels);
                Ok(is_configured)
            },
            Err(e) => {
                println!("[Backend] Recovery check error: {}", e);
                Ok(false)
            },
        }
    }

    /// Reset vault completely (delete vault file and clear memory)
    pub fn reset(&self) -> ApiResult<()> {
        // Return error if not using virtual backend - REMOVED to allow mixed mode
        // if !is_virtual_backend() {
        //     return Err(ApiError::ValidationError {
        //         message: "Vault reset is only available for Virtual File System backend".to_string()
        //     });
        // }

        // Clear VFS from memory
        {
            let mut vfs_guard = VAULT_FS.lock().unwrap();
            *vfs_guard = None;
        }

        // Clear recovery session
        {
            let mut recovery_guard = RECOVERY_SESSION.lock().unwrap();
            *recovery_guard = None;
        }

        // Get vault paths from config
        let config = APP_CONFIG.read().unwrap();
        let vault_paths = config.get_vault_paths()
            .map_err(|e| ApiError::Internal { message: e })?;

        // Delete vault files
        Self::remove_file(&vault_paths.fs_json.to_string_lossy())?;
        Self::remove_file(&vault_paths.vault_meta.to_string_lossy())?;
        Self::remove_file(&vault_paths.vault_bin.to_string_lossy())?;

        tracing::info!("Vault reset completed: files deleted from {:?}", vault_paths.dir);
        Ok(())
    }

    fn remove_file(vault_path: &str) -> Result<(), ApiError> {
        if std::path::Path::new(vault_path).exists() {
            std::fs::remove_file(vault_path)
                .map_err(|e| ApiError::Internal { message: format!("Failed to delete vault file: {}", e) })?;
        }
        Ok(())
    }

    // ==================== STEGANOGRAPHY METHODS ====================

    /// Embeds the current vault into a host file
    pub fn create_stego_container(
        &self,
        host_path: String,
        output_path: String,
        password: String,
    ) -> ApiResult<()> {
        use crate::api::steganography;
        use std::path::Path;

        // Get current vault directory
        let config = APP_CONFIG.read().unwrap();
        let vault_paths = config.get_vault_paths()
            .map_err(|e| ApiError::Internal { message: e })?;

        // Validate paths
        let host = Path::new(&host_path);
        let output = Path::new(&output_path);
        let vault_root = &vault_paths.dir;

        if !host.exists() {
            return Err(ApiError::ValidationError { message: "Host file does not exist".to_string() });
        }

        // Embed
        steganography::embed_vault(host, vault_root, output, &password)
            .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        Ok(())
    }

    /// Embeds an arbitrary file or directory into a host file
    pub fn hide_path_in_container(
        &self,
        source_path: String,
        host_path: String,
        output_path: String,
        password: String,
    ) -> ApiResult<()> {
        use crate::api::steganography;
        use std::path::Path;

        let source = Path::new(&source_path);
        let host = Path::new(&host_path);
        let output = Path::new(&output_path);

        // Basic validation
        if !host.exists() {
             return Err(ApiError::ValidationError { message: "Host file does not exist".to_string() });
        }
        
        if !source.exists() {
             return Err(ApiError::ValidationError { message: "Source file does not exist (Real FS only for now)".to_string() });
        }

        steganography::embed_path(host, source, output, &password)
             .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        Ok(())
    }

    /// Create a standalone encrypted container from a source path (file or directory).
    /// This is similar to embedding, but without a host file.
    pub fn create_container(
        &self,
        source_path: String,
        output_path: String,
        password: String,
    ) -> ApiResult<()> {
        use crate::api::steganography;
        use std::path::Path;

        let source = Path::new(&source_path);
        let output = Path::new(&output_path);

        if !source.exists() {
             return Err(ApiError::ValidationError { message: "Source path does not exist".to_string() });
        }

        steganography::create_container(source, output, &password)
             .map_err(|e| ApiError::Internal { message: e.to_string() })?;

        Ok(())
    }

    /// Create a new empty secure folder (standalone container)
    pub fn create_new_secure_folder(
        &self,
        name: String,
        parent_path: String,
        password: String,
    ) -> ApiResult<()> {
        use std::path::Path;
        use tempfile::tempdir;

        let parent = Path::new(&parent_path);
        let mut filename = name.clone();
        if !filename.ends_with(".safe") {
            filename.push_str(".safe");
        }
        let output_path = parent.join(&filename);

        if output_path.exists() {
            return Err(ApiError::ValidationError { message: format!("File already exists: {}", output_path.display()) });
        }

        // Create temp dir
        let temp_dir = tempdir().map_err(|e| ApiError::Internal { message: e.to_string() })?;
        
        // Create container from empty temp dir
        self.create_container(
            temp_dir.path().to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string(),
            password
        )?;

        Ok(())
    }

    /// Extract hidden content from a container file to a specific directory
    pub fn extract_from_container(
        &self,
        container_path: String,
        output_path: String,
        password: String,
    ) -> ApiResult<()> {
        use crate::api::steganography;
        use std::path::Path;

        let container = Path::new(&container_path);
        let output = Path::new(&output_path);

        if !container.exists() {
            return Err(ApiError::ValidationError { message: "Container file does not exist".to_string() });
        }

        // Ensure output directory exists
        if !output.exists() {
            std::fs::create_dir_all(output)
                .map_err(|e| ApiError::Internal { message: format!("Failed to create output directory: {}", e) })?;
        }

        steganography::extract_vault(container, output, &password)
            .map_err(|e| match e {
                VaultError::InvalidData => ApiError::ValidationError { message: "Invalid container or wrong password".to_string() },
                VaultError::DecryptionFailed => ApiError::ValidationError { message: "Wrong password (decryption failed)".to_string() },
                _ => ApiError::Internal { message: e.to_string() },
            })?;

        Ok(())
    }

    /// Open a vault hidden in a container file
    pub fn open_stego_container(&self, container_path: String, password: String) -> ApiResult<String> {
        use crate::api::steganography;
        use std::path::Path;
        use uuid::Uuid;

        let container = Path::new(&container_path);
        if !container.exists() {
            return Err(ApiError::ValidationError { message: "Container file does not exist".to_string() });
        }

        // Create a unique temp directory for extraction
        let temp_dir = std::env::temp_dir().join(format!("vfdir_stego_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| ApiError::Internal { message: format!("Failed to create temp dir: {}", e) })?;

        // Extract
        steganography::extract_vault(container, &temp_dir, &password)
            .map_err(|e| match e {
                VaultError::InvalidData => ApiError::ValidationError { message: "Invalid container or wrong password".to_string() },
                VaultError::DecryptionFailed => ApiError::ValidationError { message: "Wrong password (decryption failed)".to_string() },
                _ => ApiError::Internal { message: e.to_string() },
            })?;

        // Generate a unique session ID
        let session_id = format!("stego_{}", Uuid::new_v4());

        // Store session metadata for saving later
        {
            let mut stego_sessions = STEGO_SESSIONS.lock().unwrap();
            stego_sessions.insert(session_id.clone(), StegoSession {
                container_path: container.to_path_buf(),
                password: password.clone(),
                temp_dir: temp_dir.clone(),
            });
        }

        // Check if extracted content is a vault (has fs.json or vault.bin)
        let fs_json_path = temp_dir.join("fs.json");
        let vault_bin_path = temp_dir.join("vault.bin");

        if fs_json_path.exists() || vault_bin_path.exists() {
            // It's a vault -> Initialize VirtualFileSystem
            let vault_paths = crate::config::VaultPaths {
                dir: temp_dir.clone(),
                fs_json: fs_json_path,
                vault_meta: temp_dir.join("vault.meta"),
                vault_bin: vault_bin_path,
            };

            let vfs = VirtualFileSystem::new_with_paths(vault_paths.clone())
                .map_err(|e| ApiError::Internal { message: e.to_string() })?;

            if let Err(e) = vfs.unlock_vault(&password) {
                tracing::warn!("Failed to auto-unlock inner vault: {}. User might need to enter password again.", e);
            }

            let mut sessions_guard = VAULT_FS_SESSIONS.lock().unwrap();
            sessions_guard.insert(session_id.clone(), vfs);
        } else {
            // It's raw files -> Initialize TemporaryFileSystem
            tracing::info!("Initializing TemporaryFileSystem for raw stego content at {:?}", temp_dir);
            let tfs = TemporaryFileSystem::new(temp_dir);
            
            let mut sessions_guard = TEMP_FS_SESSIONS.lock().unwrap();
            sessions_guard.insert(session_id.clone(), tfs);
        }

        Ok(session_id)
    }

    /// Save the current steganography session back to the container file
    pub fn save_stego_container(&self, session_id: String) -> ApiResult<()> {
        use crate::api::steganography;

        // Retrieve session metadata
        let (container_path, password, temp_dir) = {
            let sessions = STEGO_SESSIONS.lock().unwrap();
            let session = sessions.get(&session_id)
                .ok_or_else(|| ApiError::ValidationError { message: "Session not found".to_string() })?;
            (session.container_path.clone(), session.password.clone(), session.temp_dir.clone())
        };

        // Update the container
        // Note: this takes the current content of temp_dir (which backs both Virtual and Temporary FS)
        // and re-encrypts/embeds it.
        steganography::update_container(&container_path, &temp_dir, &password)
            .map_err(|e| ApiError::Internal { message: format!("Failed to update container: {}", e) })?;

        tracing::info!("Successfully saved stego session {} to {:?}", session_id, container_path);
        Ok(())
    }
}

impl Default for VaultService {
    fn default() -> Self {
        Self::new()
    }
}
