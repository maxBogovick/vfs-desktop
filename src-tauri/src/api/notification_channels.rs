/**
 * Notification Channels for Recovery
 *
 * Extensible architecture for different notification methods:
 * - Email (implemented)
 * - Push notifications (future)
 * - SMS (future)
 * - Telegram bot (future)
 */

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Channel not configured: {0}")]
    NotConfigured(String),

    #[error("Failed to send notification: {0}")]
    SendFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

pub type ChannelResult<T> = Result<T, ChannelError>;

/// Channel type identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Email,
    Push,
    Sms,
    Telegram,
}

/// Channel configuration stored in vault.meta
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChannelConfig {
    Email {
        address: String,
        verified: bool,
    },
    Push {
        device_token: String,
        platform: String, // "ios" | "android"
    },
    Sms {
        phone_number: String,
        verified: bool,
    },
    Telegram {
        chat_id: String,
        verified: bool,
    },
}

impl ChannelConfig {
    pub fn channel_type(&self) -> ChannelType {
        match self {
            ChannelConfig::Email { .. } => ChannelType::Email,
            ChannelConfig::Push { .. } => ChannelType::Push,
            ChannelConfig::Sms { .. } => ChannelType::Sms,
            ChannelConfig::Telegram { .. } => ChannelType::Telegram,
        }
    }

    pub fn is_verified(&self) -> bool {
        match self {
            ChannelConfig::Email { verified, .. } => *verified,
            ChannelConfig::Push { .. } => true, // Push doesn't need verification
            ChannelConfig::Sms { verified, .. } => *verified,
            ChannelConfig::Telegram { verified, .. } => *verified,
        }
    }
}

/// Abstract notification channel trait
pub trait NotificationChannel {
    /// Send recovery code
    fn send_recovery_code(&self, code: &str) -> ChannelResult<()>;

    /// Get channel type
    fn channel_type(&self) -> ChannelType;

    /// Check if channel is available
    fn is_available(&self) -> bool;
}

/// Email notification channel
pub struct EmailChannel {
    config: ChannelConfig,
}

impl EmailChannel {
    pub fn new(config: ChannelConfig) -> ChannelResult<Self> {
        match &config {
            ChannelConfig::Email { verified, .. } if !verified => {
                Err(ChannelError::InvalidConfig("Email not verified".into()))
            }
            ChannelConfig::Email { .. } => Ok(Self { config }),
            _ => Err(ChannelError::InvalidConfig("Not an email config".into())),
        }
    }

    fn send_email(&self, to: &str, subject: &str, body: &str) -> ChannelResult<()> {
        // TODO: Implement actual email sending
        // Options:
        // 1. Use SMTP (lettre crate)
        // 2. Use email API (SendGrid, AWS SES, Mailgun)
        // 3. Use local mail command

        tracing::info!("Sending email to: {}", to);
        tracing::info!("Subject: {}", subject);
        tracing::info!("Body: {}", body);

        // For now, just log (development mode)
        // In production, implement actual sending
        Ok(())
    }
}

impl NotificationChannel for EmailChannel {
    fn send_recovery_code(&self, code: &str) -> ChannelResult<()> {
        if let ChannelConfig::Email { address, .. } = &self.config {
            let subject = "🔐 Vault Recovery Code";
            let body = format!(
                "Your vault recovery code is:\n\n{}\n\n\
                This code will expire in 15 minutes.\n\
                Do not share this code with anyone.\n\n\
                If you did not request this code, please ignore this email.",
                code
            );

            self.send_email(address, subject, &body)
        } else {
            Err(ChannelError::InvalidConfig("Invalid email config".into()))
        }
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Email
    }

    fn is_available(&self) -> bool {
        // Check if email service is available
        // For now, always return true
        true
    }
}

/// Push notification channel (future implementation)
pub struct PushChannel {
    config: ChannelConfig,
}

impl PushChannel {
    pub fn new(config: ChannelConfig) -> ChannelResult<Self> {
        match config {
            ChannelConfig::Push { .. } => Ok(Self { config }),
            _ => Err(ChannelError::InvalidConfig("Not a push config".into())),
        }
    }
}

impl NotificationChannel for PushChannel {
    fn send_recovery_code(&self, _code: &str) -> ChannelResult<()> {
        // TODO: Implement push notifications
        // Options: FCM (Firebase), APNs (Apple), OneSignal
        Err(ChannelError::SendFailed("Push notifications not implemented yet".into()))
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Push
    }

    fn is_available(&self) -> bool {
        false // Not implemented yet
    }
}

/// Factory for creating notification channels
pub fn create_channel(config: ChannelConfig) -> ChannelResult<Box<dyn NotificationChannel>> {
    match config.channel_type() {
        ChannelType::Email => {
            let channel = EmailChannel::new(config)?;
            Ok(Box::new(channel))
        }
        ChannelType::Push => {
            let channel = PushChannel::new(config)?;
            Ok(Box::new(channel))
        }
        _ => Err(ChannelError::NotConfigured(format!(
            "Channel type {:?} not implemented",
            config.channel_type()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_channel_config() {
        let config = ChannelConfig::Email {
            address: "user@example.com".to_string(),
            verified: true,
        };

        assert_eq!(config.channel_type(), ChannelType::Email);
        assert!(config.is_verified());
    }

    #[test]
    fn test_create_email_channel() {
        let config = ChannelConfig::Email {
            address: "user@example.com".to_string(),
            verified: true,
        };

        let channel = create_channel(config);
        assert!(channel.is_ok());
    }

    #[test]
    fn test_unverified_email_rejected() {
        let config = ChannelConfig::Email {
            address: "user@example.com".to_string(),
            verified: false,
        };

        let result = EmailChannel::new(config);
        assert!(result.is_err());
    }
}
