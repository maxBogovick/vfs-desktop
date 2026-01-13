/**
 * API Service Error Types
 *
 * Professional error handling with thiserror for ergonomic error definitions.
 */

use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Main error type for API operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ApiError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Invalid path: {path}")]
    InvalidPath { path: String },

    #[error("Operation failed: {message}")]
    OperationFailed { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Already exists: {resource}")]
    AlreadyExists { resource: String },

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("System error: {0}")]
    System(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl ApiError {
    /// Get error code for clients
    pub fn code(&self) -> &'static str {
        match self {
            ApiError::FileNotFound { .. } => "FILE_NOT_FOUND",
            ApiError::PermissionDenied { .. } => "PERMISSION_DENIED",
            ApiError::InvalidPath { .. } => "INVALID_PATH",
            ApiError::OperationFailed { .. } => "OPERATION_FAILED",
            ApiError::ValidationError { .. } => "VALIDATION_ERROR",
            ApiError::IoError { .. } => "IO_ERROR",
            ApiError::NotFound { .. } => "NOT_FOUND",
            ApiError::AlreadyExists { .. } => "ALREADY_EXISTS",
            ApiError::Internal { .. } => "INTERNAL_ERROR",
            ApiError::System(_) => "SYSTEM_ERROR",
            ApiError::Validation(_) => "VALIDATION_ERROR",
        }
    }

    /// Convert to user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            ApiError::FileNotFound { path } => {
                format!("The file or folder '{}' could not be found", Self::basename(path))
            }
            ApiError::PermissionDenied { path } => {
                format!("You don't have permission to access '{}'", Self::basename(path))
            }
            ApiError::InvalidPath { path } => {
                format!("The path '{}' is not valid", path)
            }
            _ => self.to_string(),
        }
    }

    fn basename(path: &str) -> &str {
        path.rsplit('/').next().unwrap_or(path)
    }
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Convert from std::io::Error
impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind;

        match err.kind() {
            ErrorKind::NotFound => ApiError::FileNotFound {
                path: "unknown".to_string(),
            },
            ErrorKind::PermissionDenied => ApiError::PermissionDenied {
                path: "unknown".to_string(),
            },
            _ => ApiError::IoError {
                message: err.to_string(),
            },
        }
    }
}

/// Convert from filesystem errors
impl From<crate::error::FsError> for ApiError {
    fn from(err: crate::error::FsError) -> Self {
        ApiError::OperationFailed {
            message: format!("{:?}", err),
        }
    }
}

/// Convert to String for Tauri commands
impl From<ApiError> for String {
    fn from(err: ApiError) -> String {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = ApiError::FileNotFound {
            path: "/test".to_string(),
        };
        assert_eq!(err.code(), "FILE_NOT_FOUND");
    }

    #[test]
    fn test_user_messages() {
        let err = ApiError::FileNotFound {
            path: "/path/to/file.txt".to_string(),
        };
        let msg = err.user_message();
        assert!(msg.contains("file.txt"));
    }
}
