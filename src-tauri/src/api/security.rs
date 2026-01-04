use zeroize::{Zeroize, ZeroizeOnDrop};
use serde::{Deserialize, Serialize};
use argon2::{Argon2, Algorithm, Version, Params};
use sha2::{Sha256, Digest};
use subtle::ConstantTimeEq;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::{Rng, rngs::OsRng};
use std::path::Path;
use std::io::Write;
use tempfile::NamedTempFile;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use super::vault_error::{VaultError, VaultResult};

// Argon2id constants
const ARGON_M_COST: u32 = 65536; // 64 MB
const ARGON_T_COST: u32 = 2;     // 2 iterations
const ARGON_P_COST: u32 = 1;     // 1 thread (parallelism)
const SALT_LENGTH: usize = 16;   // 16 bytes for salt
const NONCE_LENGTH: usize = 12;  // 12 bytes for AES-GCM nonce

/// Configuration saved in vault.meta
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VaultConfig {
    pub version: u8,
    pub kdf_salt: String,                  // Base64 encoded salt
    pub auth_verification_hash: String,    // Base64 SHA256(MasterKey)
    pub encryption_algo: String,           // "AES-256-GCM"

    #[serde(default)]
    pub recovery: Option<crate::api::recovery::RecoveryConfig>,
}

impl VaultConfig {
    pub fn new(kdf_salt: String, auth_verification_hash: String) -> Self {
        Self {
            version: 1,
            kdf_salt,
            auth_verification_hash,
            encryption_algo: "AES-256-GCM".to_string(),
            recovery: None,
        }
    }

    pub fn with_recovery(mut self, recovery: crate::api::recovery::RecoveryConfig) -> Self {
        self.recovery = Some(recovery);
        self
    }
}

/// Session in memory. ZeroizeOnDrop ensures key is zeroed on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct VaultSession {
    #[zeroize(skip)]
    _marker: (),
    pub master_key: [u8; 32],
}

impl VaultSession {
    pub fn new(master_key: [u8; 32]) -> Self {
        Self {
            _marker: (),
            master_key,
        }
    }
}

/// Global VFS status enum
pub enum VfsStatus {
    /// Vault not created (first run)
    NotInitialized,
    /// Vault exists but key is not in memory
    Locked,
    /// Vault is unlocked, filesystem is accessible
    Unlocked {
        fs: crate::api::virtual_fs::VfsState,
        session: VaultSession,
    },
}

// ==================== KEY DERIVATION ====================

/// Generate a random salt for Argon2
pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_LENGTH];
    OsRng.fill(&mut salt[..]);
    salt
}

/// Derive master key from password using Argon2id
pub fn derive_master_key(password: &str, salt: &[u8]) -> VaultResult<VaultSession> {
    let mut key = [0u8; 32];

    let params = Params::new(
        ARGON_M_COST,
        ARGON_T_COST,
        ARGON_P_COST,
        Some(32)
    ).map_err(|e| VaultError::CryptoError(format!("Argon2 params error: {}", e)))?;

    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    argon.hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| VaultError::CryptoError(format!("KDF failed: {}", e)))?;

    Ok(VaultSession::new(key))
}

/// Create verification hash from master key (SHA256)
pub fn create_verification_hash(session: &VaultSession) -> String {
    let mut hasher = Sha256::new();
    hasher.update(&session.master_key);
    let hash = hasher.finalize();
    BASE64.encode(&hash[..])
}

/// Verify master key against stored hash (constant-time comparison)
pub fn verify_key(session: &VaultSession, expected_hash: &str) -> VaultResult<bool> {
    let mut hasher = Sha256::new();
    hasher.update(&session.master_key);
    let hash = hasher.finalize();

    let expected_bytes = BASE64.decode(expected_hash)
        .map_err(|e| VaultError::Base64Error(e))?;

    if expected_bytes.len() != hash.len() {
        return Ok(false);
    }

    Ok(bool::from(hash.ct_eq(&expected_bytes[..])))
}

// ==================== ENCRYPTION/DECRYPTION ====================

/// Encrypt data blob using AES-256-GCM
/// Returns: [Nonce (12 bytes)] + [Ciphertext + AuthTag]
pub fn encrypt_blob(data: &[u8], session: &VaultSession) -> VaultResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(&session.master_key.into());

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt with authenticated encryption
    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|_| VaultError::EncryptionFailed)?;

    // Build final blob: [Nonce] + [Ciphertext + AuthTag]
    let mut blob = nonce_bytes.to_vec();
    blob.extend_from_slice(&ciphertext);

    Ok(blob)
}

/// Decrypt data blob using AES-256-GCM
/// Expects: [Nonce (12 bytes)] + [Ciphertext + AuthTag]
pub fn decrypt_blob(blob: &[u8], session: &VaultSession) -> VaultResult<Vec<u8>> {
    if blob.len() < NONCE_LENGTH {
        return Err(VaultError::InvalidData);
    }

    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LENGTH);
    let cipher = Aes256Gcm::new(&session.master_key.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt and verify authentication tag
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| VaultError::DecryptionFailed)
}

// ==================== ATOMIC FILE OPERATIONS ====================

/// Atomic write using temp file + rename
/// This prevents data corruption on power failure
pub fn atomic_write(path: &Path, data: &[u8]) -> VaultResult<()> {
    // Get parent directory for temp file
    let dir = path.parent().ok_or(VaultError::InvalidPath)?;

    // Create temp file in the SAME directory (required for atomic rename)
    let mut temp_file = NamedTempFile::new_in(dir)?;

    // Write data
    temp_file.write_all(data)?;
    temp_file.flush()?; // Force flush to disk

    // Atomic rename (if power fails before this, original file is untouched)
    temp_file.persist(path)
        .map_err(|e| VaultError::Io(e.error))?;

    Ok(())
}

// ==================== TOML HELPERS ====================

/// Save vault config to TOML file
pub fn save_vault_config(path: &Path, config: &VaultConfig) -> VaultResult<()> {
    let toml_string = toml::to_string_pretty(config)
        .map_err(|e| VaultError::Serialization(format!("TOML serialization failed: {}", e)))?;

    atomic_write(path, toml_string.as_bytes())
}

/// Load vault config from TOML file
pub fn load_vault_config(path: &Path) -> VaultResult<VaultConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: VaultConfig = toml::from_str(&content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let salt = generate_salt();
        let password = "test_password_123";

        let session = derive_master_key(password, &salt).unwrap();
        assert_eq!(session.master_key.len(), 32);
    }

    #[test]
    fn test_verification_hash() {
        let salt = generate_salt();
        let password = "test_password";

        let session = derive_master_key(password, &salt).unwrap();
        let hash = create_verification_hash(&session);

        assert!(verify_key(&session, &hash).unwrap());
    }

    #[test]
    fn test_encryption_decryption() {
        let salt = generate_salt();
        let password = "test_password";
        let session = derive_master_key(password, &salt).unwrap();

        let plaintext = b"Hello, secure world!";
        let encrypted = encrypt_blob(plaintext, &session).unwrap();
        let decrypted = decrypt_blob(&encrypted, &session).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_wrong_key_fails() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        let password = "test_password";

        let session1 = derive_master_key(password, &salt1).unwrap();
        let session2 = derive_master_key(password, &salt2).unwrap();

        let plaintext = b"Secret data";
        let encrypted = encrypt_blob(plaintext, &session1).unwrap();

        // Trying to decrypt with wrong key should fail
        assert!(decrypt_blob(&encrypted, &session2).is_err());
    }
}
