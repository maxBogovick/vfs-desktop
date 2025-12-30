/**
 * Bookmarks Service
 *
 * Manages user bookmarks/favorites with persistence.
 */

use super::{ApiResult, ApiError};
use super::models::Bookmark;
use crate::state::APP_CONFIG;

pub struct BookmarkService {}

impl BookmarkService {
    pub fn new() -> Self {
        tracing::debug!("Initializing BookmarkService");
        Self {}
    }

    /// Get all bookmarks
    pub fn get_all(&self) -> ApiResult<Vec<Bookmark>> {
        tracing::debug!("Fetching all bookmarks");
        let config = APP_CONFIG.read().unwrap();
        Ok(config.bookmarks.clone())
    }

    /// Add new bookmark
    ///
    /// # Arguments
    /// * `path` - Path to bookmark
    /// * `name` - Optional custom name (uses folder name if not provided)
    pub fn add(&self, path: String, name: Option<String>) -> ApiResult<Bookmark> {
        tracing::info!("Adding bookmark: {}", path);
        let bookmark_name = name.unwrap_or_else(|| {
            path.split('/').last().unwrap_or(&path).to_string()
        });

        let bookmark = Bookmark {
            id: format!("bookmark-{}", uuid::Uuid::new_v4()),
            name: bookmark_name,
            path,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let mut config = APP_CONFIG.write().unwrap();
        config.bookmarks.push(bookmark.clone());
        drop(config);

        // Save to file
        let config = APP_CONFIG.read().unwrap();
        config.save().map_err(|e| {
            tracing::error!("Failed to save config: {}", e);
            ApiError::OperationFailed {
                message: format!("Failed to save bookmark: {}", e),
            }
        })?;

        tracing::info!("Bookmark added successfully: {}", bookmark.id);
        Ok(bookmark)
    }

    /// Remove bookmark by ID
    pub fn remove(&self, id: &str) -> ApiResult<()> {
        tracing::info!("Removing bookmark: {}", id);
        let mut config = APP_CONFIG.write().unwrap();
        config.bookmarks.retain(|b| b.id != id);
        drop(config);

        let config = APP_CONFIG.read().unwrap();
        config.save().map_err(|e| {
            tracing::error!("Failed to save config: {}", e);
            ApiError::OperationFailed {
                message: format!("Failed to remove bookmark: {}", e),
            }
        })?;

        Ok(())
    }

    /// Rename bookmark
    pub fn rename(&self, id: &str, new_name: String) -> ApiResult<()> {
        tracing::info!("Renaming bookmark {}: {}", id, new_name);
        let mut config = APP_CONFIG.write().unwrap();

        if let Some(bookmark) = config.bookmarks.iter_mut().find(|b| b.id == id) {
            bookmark.name = new_name;
        } else {
            tracing::warn!("Bookmark not found: {}", id);
            return Err(ApiError::NotFound {
                resource: format!("Bookmark with ID '{}'", id),
            });
        }

        drop(config);

        let config = APP_CONFIG.read().unwrap();
        config.save().map_err(|e| {
            tracing::error!("Failed to save config: {}", e);
            ApiError::OperationFailed {
                message: format!("Failed to rename bookmark: {}", e),
            }
        })?;

        Ok(())
    }
}

impl Default for BookmarkService {
    fn default() -> Self {
        Self::new()
    }
}
