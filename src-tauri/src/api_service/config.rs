/**
 * Configuration Service
 *
 * Manages application configuration and UI state persistence.
 */

use super::{ApiResult, ApiError};
use super::models::{AppConfig, UIState};
use crate::state::APP_CONFIG;

pub struct ConfigService {}

impl ConfigService {
    pub fn new() -> Self {
        tracing::debug!("Initializing ConfigService");
        Self {}
    }

    /// Get current application configuration
    pub fn get(&self) -> ApiResult<AppConfig> {
        tracing::debug!("Fetching application config");
        let config = APP_CONFIG.read().unwrap();
        Ok(config.clone())
    }

    /// Update entire configuration
    pub fn update(&self, new_config: AppConfig) -> ApiResult<()> {
        tracing::info!("Updating application configuration");

        new_config.save().map_err(|e| {
            tracing::error!("Failed to save config: {}", e);
            ApiError::OperationFailed {
                message: format!("Failed to update config: {}", e),
            }
        })?;

        let mut config = APP_CONFIG.write().unwrap();
        *config = new_config;

        Ok(())
    }

    /// Get UI state
    pub fn get_ui_state(&self) -> ApiResult<UIState> {
        let config = APP_CONFIG.read().unwrap();
        Ok(config.ui_state.clone())
    }

    /// Save UI state
    pub fn save_ui_state(&self, ui_state: UIState) -> ApiResult<()> {
        tracing::debug!("Saving UI state");

        let mut config = APP_CONFIG.write().unwrap();
        config.ui_state = ui_state;
        drop(config);

        let config = APP_CONFIG.read().unwrap();
        config.save().map_err(|e| {
            tracing::error!("Failed to save UI state: {}", e);
            ApiError::OperationFailed {
                message: format!("Failed to save UI state: {}", e),
            }
        })?;

        Ok(())
    }

    /// Set filesystem backend (real or virtual)
    pub fn set_filesystem_backend(&self, backend: &str) -> ApiResult<()> {
        use crate::config::FileSystemBackend;

        tracing::info!("Setting filesystem backend to: {}", backend);

        let backend_enum = match backend {
            "real" => FileSystemBackend::Real,
            "virtual" => FileSystemBackend::Virtual,
            _ => {
                return Err(ApiError::ValidationError {
                    message: format!("Invalid backend type: '{}'. Must be 'real' or 'virtual'", backend),
                })
            }
        };

        let mut config = APP_CONFIG.write().unwrap();
        config.filesystem_backend = backend_enum;
        drop(config);

        let config = APP_CONFIG.read().unwrap();
        config.save().map_err(|e| {
            tracing::error!("Failed to save backend config: {}", e);
            ApiError::OperationFailed {
                message: format!("Failed to save config: {}", e),
            }
        })?;

        Ok(())
    }
}

impl Default for ConfigService {
    fn default() -> Self {
        Self::new()
    }
}
