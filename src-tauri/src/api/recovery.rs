/**
 * Password Recovery System
 *
 * Architecture:
 * 1. Recovery Key (256-bit) generated at vault creation
 * 2. Recovery Key encrypted with master password (stored locally)
 * 3. Recovery Key encrypted with temporary verification code (sent via channel)
 * 4. User receives code, decrypts recovery key, sets new password
 */

use super::notification_channels::{ChannelConfig, create_channel, ChannelResult};
use super::security::{VaultSession, derive_master_key, create_verification_hash, encrypt_blob, decrypt_blob};
use super::vault_error::{VaultError, VaultResult};
use rand::{Rng, rngs::OsRng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Recovery configuration stored in vault.meta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Recovery key encrypted with master password
    pub encrypted_recovery_key: String, // Base64

    /// Configured notification channels
    pub channels: Vec<ChannelConfig>,

    /// Recovery attempts tracking
    pub last_recovery_attempt: Option<u64>, // Unix timestamp
    pub recovery_attempts_count: u32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            encrypted_recovery_key: String::new(),
            channels: Vec::new(),
            last_recovery_attempt: None,
            recovery_attempts_count: 0,
        }
    }
}

/// Recovery key (256-bit random key)
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct RecoveryKey {
    key: [u8; 32],
}

impl RecoveryKey {
    /// Generate new random recovery key
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill(&mut key);
        Self { key }
    }

    /// Get key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> VaultResult<Self> {
        if bytes.len() != 32 {
            return Err(VaultError::CryptoError("Invalid recovery key length".into()));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(Self { key })
    }

    /// Encode as human-readable format (24 words BIP39-style)
    pub fn to_mnemonic(&self) -> String {
        // TODO: Implement BIP39 encoding
        // For now, use base64
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&self.key)
    }

    /// Decode from mnemonic
    pub fn from_mnemonic(mnemonic: &str) -> VaultResult<Self> {
        // TODO: Implement BIP39 decoding
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(mnemonic)
            .map_err(|e| VaultError::Base64Error(e))?;
        Self::from_bytes(&bytes)
    }
}

/// Active recovery session (temporary, in-memory only)
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct RecoverySession {
    /// Verification code sent to user
    verification_code: String,

    /// Encrypted recovery key (with verification code)
    encrypted_recovery_key: Vec<u8>,

    /// Expiration timestamp
    expires_at: u64,

    /// Salt for code derivation
    code_salt: [u8; 16],
}

impl RecoverySession {
    /// Create new recovery session
    pub fn new(recovery_key: &RecoveryKey) -> VaultResult<Self> {
        // Generate random verification code (6 digits)
        let verification_code = Self::generate_verification_code();

        // Generate salt for code derivation
        let mut code_salt = [0u8; 16];
        OsRng.fill(&mut code_salt);

        // Derive temporary key from verification code
        let temp_session = derive_master_key(&verification_code, &code_salt)?;

        // Encrypt recovery key with temp session
        let encrypted_recovery_key = encrypt_blob(recovery_key.as_bytes(), &temp_session)?;

        // Set expiration (15 minutes)
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 900; // 15 minutes

        Ok(Self {
            verification_code,
            encrypted_recovery_key,
            expires_at,
            code_salt,
        })
    }

    /// Generate 6-digit verification code
    fn generate_verification_code() -> String {
        let code: u32 = OsRng.gen_range(100000..999999);
        code.to_string()
    }

    /// Get verification code (to send via notification)
    pub fn verification_code(&self) -> &str {
        &self.verification_code
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }

    /// Verify code and decrypt recovery key
    pub fn verify_and_decrypt(&self, code: &str) -> VaultResult<RecoveryKey> {
        if self.is_expired() {
            return Err(VaultError::CryptoError("Recovery session expired".into()));
        }

        if code != self.verification_code {
            return Err(VaultError::InvalidPassword);
        }

        // Derive temp session from code
        let temp_session = derive_master_key(code, &self.code_salt)?;

        // Decrypt recovery key
        let decrypted = decrypt_blob(&self.encrypted_recovery_key, &temp_session)?;

        RecoveryKey::from_bytes(&decrypted)
    }

    /// Export session data for sending
    pub fn export_encrypted_data(&self) -> String {
        use base64::Engine;
        // Return base64 encoded encrypted recovery key + salt
        let mut data = self.code_salt.to_vec();
        data.extend_from_slice(&self.encrypted_recovery_key);
        ::base64::engine::general_purpose::STANDARD.encode(&data)
    }
}

/// Recovery manager
pub struct RecoveryManager;

impl RecoveryManager {
    /// Setup recovery for a vault
    pub fn setup_recovery(
        master_session: &VaultSession,
        channels: Vec<ChannelConfig>,
    ) -> VaultResult<(RecoveryKey, RecoveryConfig)> {
        use base64::Engine;
        // Generate recovery key
        let recovery_key = RecoveryKey::generate();

        // Encrypt recovery key with master password
        let encrypted = encrypt_blob(recovery_key.as_bytes(), master_session)?;
        let encrypted_recovery_key = ::base64::engine::general_purpose::STANDARD.encode(&encrypted);

        let config = RecoveryConfig {
            encrypted_recovery_key,
            channels,
            last_recovery_attempt: None,
            recovery_attempts_count: 0,
        };

        Ok((recovery_key, config))
    }

    /// Initiate password recovery
    pub fn initiate_recovery(
        recovery_config: &RecoveryConfig,
        channel_type: &str, // "email", "push", etc.
    ) -> VaultResult<RecoverySession> {
        // Check rate limiting (max 3 attempts per hour)
        if let Some(last_attempt) = recovery_config.last_recovery_attempt {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now - last_attempt < 3600 && recovery_config.recovery_attempts_count >= 3 {
                return Err(VaultError::CryptoError("Too many recovery attempts. Try again later.".into()));
            }
        }

        // Find channel
        let channel_config = recovery_config
            .channels
            .iter()
            .find(|c| format!("{:?}", c.channel_type()).to_lowercase() == channel_type)
            .ok_or_else(|| VaultError::CryptoError(format!("Channel {} not configured", channel_type)))?;

        // Check if channel is verified
        if !channel_config.is_verified() {
            return Err(VaultError::CryptoError("Channel not verified".into()));
        }

        // Decrypt recovery key with placeholder (we don't have master password)
        // Instead, we'll create a new recovery session
        let recovery_key = RecoveryKey::generate(); // Temporary, will be replaced
        let session = RecoverySession::new(&recovery_key)?;

        // Send verification code via channel
        let channel = create_channel(channel_config.clone())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        channel.send_recovery_code(session.verification_code())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        Ok(session)
    }

    /// Complete recovery with new password
    pub fn complete_recovery(
        recovery_key: &RecoveryKey,
        new_password: &str,
        salt: &[u8],
    ) -> VaultResult<(VaultSession, String)> {
        // Derive new master key from new password
        let new_session = derive_master_key(new_password, salt)?;

        // Create new verification hash
        let new_hash = create_verification_hash(&new_session);

        Ok((new_session, new_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_key_generation() {
        let key1 = RecoveryKey::generate();
        let key2 = RecoveryKey::generate();

        // Keys should be different
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_recovery_session() {
        let recovery_key = RecoveryKey::generate();
        let session = RecoverySession::new(&recovery_key).unwrap();

        // Code should be 6 digits
        assert_eq!(session.verification_code().len(), 6);

        // Should not be expired immediately
        assert!(!session.is_expired());

        // Should decrypt with correct code
        let code = session.verification_code().to_string();
        let decrypted = session.verify_and_decrypt(&code).unwrap();
        assert_eq!(decrypted.as_bytes(), recovery_key.as_bytes());
    }

    #[test]
    fn test_wrong_code_fails() {
        let recovery_key = RecoveryKey::generate();
        let session = RecoverySession::new(&recovery_key).unwrap();

        let result = session.verify_and_decrypt("000000");
        assert!(result.is_err());
    }
}
