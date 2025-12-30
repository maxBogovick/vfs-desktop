/**
 * API Service Models
 *
 * Domain models and DTOs for API services.
 * These are framework-agnostic and can be serialized for any transport.
 */

use serde::{Deserialize, Serialize};

// Re-export core types
pub use crate::core::FileSystemEntry;
pub use crate::config::{Bookmark, AppConfig, UIState};
pub use super::error::{ApiError, ApiResult};

/// Batch Rename Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRenameRequest {
    pub files: Vec<String>,
    pub patterns: Vec<RenamePattern>,
    pub apply_to_folders: bool,
    pub apply_to_files: bool,
    pub preserve_extension: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RenamePattern {
    Prefix { enabled: bool, text: String },
    Suffix {
        enabled: bool,
        text: String,
        before_extension: bool,
    },
    Replace {
        enabled: bool,
        search_text: String,
        replace_text: String,
        case_sensitive: bool,
        whole_word: bool,
    },
    Regex {
        enabled: bool,
        pattern: String,
        replacement: String,
        flags: String,
    },
    Numbering {
        enabled: bool,
        start_number: i32,
        increment: i32,
        padding: usize,
        position: String,
        separator: String,
    },
    Case {
        enabled: bool,
        case_type: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamePreview {
    pub original_name: String,
    pub new_name: String,
    pub original_path: String,
    pub has_error: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRenameResult {
    pub operation_id: String,
    pub previews: Vec<RenamePreview>,
}

/// Batch Attribute Change Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAttributeRequest {
    pub files: Vec<String>,
    pub permissions: Option<PermissionsChange>,
    pub dates: Option<DateChange>,
    pub tags: Option<TagsChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsChange {
    pub readable: Option<bool>,
    pub writable: Option<bool>,
    pub executable: Option<bool>,
    pub recursive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateChange {
    pub modified: Option<u64>,
    pub created: Option<u64>,
    pub accessed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsChange {
    pub operation: String, // "add" | "remove" | "replace"
    pub tags: Vec<String>,
}

/// System Stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub memory_mb: f64,
    pub cpu_percent: f32,
}
