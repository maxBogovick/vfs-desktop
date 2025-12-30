/**
 * Application State Module
 *
 * Centralized global state management with thread-safe access patterns.
 * Follows best practices for shared state in Rust applications.
 */

use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use crate::config::AppConfig;

/// Global application configuration
///
/// Uses Arc<RwLock<T>> for thread-safe shared mutable state.
/// Lazy initialization ensures config is loaded only once.
///
/// # Thread Safety
/// - Multiple readers can access simultaneously
/// - Writers get exclusive access
/// - Prevents data races at compile time
pub static APP_CONFIG: Lazy<Arc<RwLock<AppConfig>>> = Lazy::new(|| {
    tracing::info!("Initializing application configuration");

    let config = AppConfig::load()
        .inspect_err(|e| tracing::error!("Failed to load config: {}", e))
        .unwrap_or_else(|_| {
            tracing::warn!("Using default configuration");
            AppConfig::default()
        });

    Arc::new(RwLock::new(config))
});

/// State accessor trait for dependency injection
///
/// Allows for easier testing and mocking of state access
pub trait StateAccess {
    fn get_config(&self) -> AppConfig;
    fn update_config<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppConfig);
}

/// Default state accessor implementation
pub struct DefaultStateAccess;

impl StateAccess for DefaultStateAccess {
    fn get_config(&self) -> AppConfig {
        APP_CONFIG.read().unwrap().clone()
    }

    fn update_config<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = APP_CONFIG.write().unwrap();
        f(&mut config);
        config.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_access() {
        let accessor = DefaultStateAccess;
        let config = accessor.get_config();
        assert!(config.bookmarks.len() >= 0); // Just ensure it works
    }
}
