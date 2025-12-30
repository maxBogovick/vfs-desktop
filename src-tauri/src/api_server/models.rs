/**
 * API Models
 *
 * Request and response models for REST API
 */

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export common types
pub use crate::core::FileSystemEntry;
pub use crate::config::{Bookmark, AppConfig, UIState};

// ===== Error Responses =====

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
            },
        }
    }

    pub fn file_not_found(path: &str) -> Self {
        Self::new("FILE_NOT_FOUND", format!("File not found: {}", path))
    }

    pub fn permission_denied(path: &str) -> Self {
        Self::new("PERMISSION_DENIED", format!("Permission denied: {}", path))
    }

    pub fn invalid_path(path: &str) -> Self {
        Self::new("INVALID_PATH", format!("Invalid path: {}", path))
    }

    pub fn operation_failed(message: impl Into<String>) -> Self {
        Self::new("OPERATION_FAILED", message)
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }
}

// ===== File Operations =====

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListDirectoryQuery {
    pub path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListDirectoryResponse {
    pub files: Vec<FileSystemEntry>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct GetFileInfoQuery {
    pub path: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFolderRequest {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CopyItemsRequest {
    pub sources: Vec<String>,
    pub destination: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MoveItemsRequest {
    pub sources: Vec<String>,
    pub destination: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RenameItemRequest {
    #[serde(rename = "oldPath")]
    pub old_path: String,
    #[serde(rename = "newName")]
    pub new_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteItemsRequest {
    pub paths: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CopyItemsWithProgressRequest {
    pub sources: Vec<String>,
    pub destination: String,
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MoveItemsWithProgressRequest {
    pub sources: Vec<String>,
    pub destination: String,
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteItemsWithProgressRequest {
    pub paths: Vec<String>,
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReadFileContentQuery {
    pub path: String,
    #[serde(rename = "maxSize")]
    pub max_size: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadFileContentResponse {
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OpenFileRequest {
    pub path: String,
}

// ===== Batch Operations =====

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BatchRenameRequest {
    pub files: Vec<String>,
    pub config: BatchRenameConfig,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BatchRenameConfig {
    pub patterns: Vec<RenamePattern>,
    #[serde(rename = "applyToFolders")]
    pub apply_to_folders: bool,
    #[serde(rename = "applyToFiles")]
    pub apply_to_files: bool,
    #[serde(rename = "preserveExtension")]
    pub preserve_extension: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum RenamePattern {
    #[serde(rename = "prefix")]
    Prefix { enabled: bool, text: String },
    #[serde(rename = "suffix")]
    Suffix {
        enabled: bool,
        text: String,
        #[serde(rename = "beforeExtension")]
        before_extension: bool,
    },
    #[serde(rename = "replace")]
    Replace {
        enabled: bool,
        #[serde(rename = "searchText")]
        search_text: String,
        #[serde(rename = "replaceText")]
        replace_text: String,
        #[serde(rename = "caseSensitive")]
        case_sensitive: bool,
        #[serde(rename = "wholeWord")]
        whole_word: bool,
    },
    #[serde(rename = "regex")]
    Regex {
        enabled: bool,
        pattern: String,
        replacement: String,
        flags: String,
    },
    #[serde(rename = "numbering")]
    Numbering {
        enabled: bool,
        #[serde(rename = "startNumber")]
        start_number: i32,
        increment: i32,
        padding: usize,
        position: String,
        separator: String,
    },
    #[serde(rename = "case")]
    Case {
        enabled: bool,
        #[serde(rename = "caseType")]
        case_type: String,
    },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchRenameResponse {
    #[serde(rename = "operationId")]
    pub operation_id: String,
    pub preview: Vec<RenamePreviewItem>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RenamePreviewItem {
    #[serde(rename = "originalName")]
    pub original_name: String,
    #[serde(rename = "newName")]
    pub new_name: String,
    #[serde(rename = "hasError")]
    pub has_error: bool,
    #[serde(rename = "errorMessage", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchAttributeRequest {
    pub files: Vec<String>,
    pub changes: BatchAttributeChanges,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchAttributeChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<PermissionsChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dates: Option<DateChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<TagsChange>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PermissionsChange {
    pub readable: Option<bool>,
    pub writable: Option<bool>,
    pub executable: Option<bool>,
    pub recursive: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DateChange {
    pub modified: Option<u64>,
    pub created: Option<u64>,
    pub accessed: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TagsChange {
    pub operation: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchAttributeResponse {
    #[serde(rename = "operationId")]
    pub operation_id: String,
}

// ===== Bookmarks =====

#[derive(Debug, Serialize, ToSchema)]
pub struct BookmarksResponse {
    pub bookmarks: Vec<Bookmark>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddBookmarkRequest {
    pub path: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RenameBookmarkRequest {
    #[serde(rename = "newName")]
    pub new_name: String,
}

// ===== System =====

#[derive(Debug, Serialize, ToSchema)]
pub struct HomeDirectoryResponse {
    pub path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemFoldersResponse {
    pub folders: Vec<FileSystemEntry>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemStatsResponse {
    pub memory_mb: f64,
    pub cpu_percent: f32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OpenTerminalRequest {
    pub path: String,
}

// ===== WebSocket Messages =====

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "progress")]
    Progress { data: ProgressData },
    #[serde(rename = "change")]
    FileSystemChange { data: FileSystemChangeData },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProgressData {
    #[serde(rename = "operationId")]
    pub operation_id: String,
    #[serde(rename = "operationType")]
    pub operation_type: String,
    pub status: String,
    #[serde(rename = "currentBytes")]
    pub current_bytes: u64,
    #[serde(rename = "totalBytes")]
    pub total_bytes: u64,
    #[serde(rename = "currentItems")]
    pub current_items: u64,
    #[serde(rename = "totalItems")]
    pub total_items: u64,
    #[serde(rename = "currentFile")]
    pub current_file: Option<String>,
    #[serde(rename = "speedBytesPerSec")]
    pub speed_bytes_per_sec: f64,
    #[serde(rename = "etaSeconds")]
    pub eta_seconds: Option<f64>,
    #[serde(rename = "errorMessage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileSystemChangeData {
    pub path: String,
    #[serde(rename = "changeType")]
    pub change_type: String, // "created" | "modified" | "deleted"
}
