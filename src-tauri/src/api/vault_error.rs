use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault is locked")]
    Locked,

    #[error("Vault is not initialized")]
    NotInitialized,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Decryption failed (integrity check error)")]
    DecryptionFailed,

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Invalid data format")]
    InvalidData,

    #[error("Invalid path")]
    InvalidPath,

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serialization(String),

    #[error("TOML Error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Base64 Error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

// For passing errors to frontend in JSON format
#[derive(Serialize)]
pub struct VaultErrorResponse {
    pub code: String, // "LOCKED", "INVALID_PASS", etc.
    pub message: String,
}

impl From<VaultError> for VaultErrorResponse {
    fn from(error: VaultError) -> Self {
        let code = match &error {
            VaultError::Locked => "LOCKED",
            VaultError::NotInitialized => "NOT_INITIALIZED",
            VaultError::InvalidPassword => "INVALID_PASSWORD",
            VaultError::DecryptionFailed => "DECRYPTION_FAILED",
            VaultError::EncryptionFailed => "ENCRYPTION_FAILED",
            VaultError::InvalidData => "INVALID_DATA",
            VaultError::InvalidPath => "INVALID_PATH",
            VaultError::CryptoError(_) => "CRYPTO_ERROR",
            VaultError::Io(_) => "IO_ERROR",
            VaultError::Serialization(_) => "SERIALIZATION_ERROR",
            VaultError::TomlError(_) => "TOML_ERROR",
            VaultError::Base64Error(_) => "BASE64_ERROR",
        };

        VaultErrorResponse {
            code: code.to_string(),
            message: error.to_string(),
        }
    }
}

pub type VaultResult<T> = Result<T, VaultError>;
