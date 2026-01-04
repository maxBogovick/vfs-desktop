/**
 * Universal API Service Layer
 *
 * Business logic layer that can be used by:
 * - Tauri desktop commands
 * - HTTP REST API server
 * - Any other client
 *
 * This layer provides a clean, framework-agnostic interface to all functionality.
 *
 * # Architecture
 *
 * ```text
 * Clients (Tauri/HTTP/CLI) → API Service → Core Layer
 * ```
 *
 * # Design Principles
 *
 * - **Single Responsibility**: Each service handles one domain
 * - **Dependency Injection**: Services can be mocked for testing
 * - **Error Handling**: Consistent error types across all operations
 * - **Thread Safety**: All services are safe for concurrent access
 */

pub mod error;
pub mod files;
pub mod batch;
pub mod bookmarks;
pub mod system;
pub mod config;
pub mod models;
pub mod vault;

pub use error::{ApiError, ApiResult};
pub use files::FileService;
pub use batch::BatchService;
pub use bookmarks::BookmarkService;
pub use system::SystemService;
pub use config::ConfigService;
pub use vault::VaultService;

/// Main API facade that groups all services
pub struct Api {
    pub files: FileService,
    pub batch: BatchService,
    pub bookmarks: BookmarkService,
    pub system: SystemService,
    pub config: ConfigService,
    pub vault: VaultService,
}

impl Api {
    pub fn new() -> Self {
        Self {
            files: FileService::new(),
            batch: BatchService::new(),
            bookmarks: BookmarkService::new(),
            system: SystemService::new(),
            config: ConfigService::new(),
            vault: VaultService::new(),
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

/// Global API instance
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static API: Lazy<Arc<Api>> = Lazy::new(|| Arc::new(Api::new()));
