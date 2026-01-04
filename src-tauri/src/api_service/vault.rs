/**
 * Vault Security Service
 *
 * Manages encrypted virtual file system vault operations.
 */

use super::{ApiResult, ApiError};
use crate::api::virtual_fs::VirtualFileSystem;
use crate::api::vault_error::{VaultError, VaultErrorResponse};
use crate::api::recovery::{RecoveryManager, RecoverySession};
use crate::api::notification_channels::ChannelConfig;
use crate::config::FileSystemBackend;
use crate::state::APP_CONFIG;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Global VirtualFileSystem instance
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
}

impl Default for VaultService {
    fn default() -> Self {
        Self::new()
    }
}
