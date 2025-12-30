/**
 * Batch Operations Handlers
 */

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::api_server::{models::*, state::AppState};
use crate::api_service::{API, ApiError};

/// Convert api_server RenamePattern to api_service RenamePattern
fn convert_rename_pattern(pattern: RenamePattern) -> crate::api_service::models::RenamePattern {
    match pattern {
        RenamePattern::Prefix { enabled, text } => {
            crate::api_service::models::RenamePattern::Prefix { enabled, text }
        }
        RenamePattern::Suffix { enabled, text, before_extension } => {
            crate::api_service::models::RenamePattern::Suffix {
                enabled,
                text,
                before_extension,
            }
        }
        RenamePattern::Replace { enabled, search_text, replace_text, case_sensitive, whole_word } => {
            crate::api_service::models::RenamePattern::Replace {
                enabled,
                search_text,
                replace_text,
                case_sensitive,
                whole_word,
            }
        }
        RenamePattern::Regex { enabled, pattern, replacement, flags } => {
            crate::api_service::models::RenamePattern::Regex {
                enabled,
                pattern,
                replacement,
                flags,
            }
        }
        RenamePattern::Numbering { enabled, start_number, increment, padding, position, separator } => {
            crate::api_service::models::RenamePattern::Numbering {
                enabled,
                start_number,
                increment,
                padding,
                position,
                separator,
            }
        }
        RenamePattern::Case { enabled, case_type } => {
            crate::api_service::models::RenamePattern::Case {
                enabled,
                case_type,
            }
        }
    }
}

pub async fn batch_rename(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BatchRenameRequest>,
) -> impl IntoResponse {
    // Convert api_server model to api_service model
    let service_req = crate::api_service::models::BatchRenameRequest {
        files: req.files,
        patterns: req.config.patterns.into_iter().map(convert_rename_pattern).collect(),
        apply_to_folders: req.config.apply_to_folders,
        apply_to_files: req.config.apply_to_files,
        preserve_extension: req.config.preserve_extension,
    };

    match API.batch.queue_rename(&service_req) {
        Ok(result) => Json(BatchRenameResponse {
            operation_id: result.operation_id,
            preview: result.previews.into_iter().map(|p| RenamePreviewItem {
                original_name: p.original_name,
                new_name: p.new_name,
                has_error: p.has_error,
                error_message: p.error_message,
            }).collect(),
        }).into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                ApiError::OperationFailed { .. } => StatusCode::NOT_IMPLEMENTED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

pub async fn batch_rename_preview(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BatchRenameRequest>,
) -> impl IntoResponse {
    // Convert api_server model to api_service model
    let service_req = crate::api_service::models::BatchRenameRequest {
        files: req.files,
        patterns: req.config.patterns.into_iter().map(convert_rename_pattern).collect(),
        apply_to_folders: req.config.apply_to_folders,
        apply_to_files: req.config.apply_to_files,
        preserve_extension: req.config.preserve_extension,
    };

    match API.batch.preview_rename(&service_req) {
        Ok(preview) => Json(preview).into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                ApiError::OperationFailed { .. } => StatusCode::NOT_IMPLEMENTED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

pub async fn batch_attributes(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BatchAttributeRequest>,
) -> impl IntoResponse {
    // Convert api_server model to api_service model
    let service_req = crate::api_service::models::BatchAttributeRequest {
        files: req.files,
        permissions: req.changes.permissions.map(|p| crate::api_service::models::PermissionsChange {
            readable: p.readable,
            writable: p.writable,
            executable: p.executable,
            recursive: p.recursive,
        }),
        dates: req.changes.dates.map(|d| crate::api_service::models::DateChange {
            modified: d.modified,
            created: d.created,
            accessed: d.accessed,
        }),
        tags: req.changes.tags.map(|t| crate::api_service::models::TagsChange {
            operation: t.operation,
            tags: t.tags,
        }),
    };

    match API.batch.queue_attribute_change(&service_req) {
        Ok(operation_id) => Json(BatchAttributeResponse { operation_id }).into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                ApiError::OperationFailed { .. } => StatusCode::NOT_IMPLEMENTED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

pub async fn get_operations(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match API.batch.get_operations() {
        Ok(operations) => Json(operations).into_response(),
        Err(err) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

pub async fn get_operation(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match API.batch.get_operation(&id) {
        Ok(operation) => Json(operation).into_response(),
        Err(err) => {
            let status = match err {
                ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

pub async fn cancel_operation(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match API.batch.cancel_operation(&id) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

pub async fn retry_operation(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match API.batch.retry_operation(&id) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
                ApiError::OperationFailed { .. } => StatusCode::NOT_IMPLEMENTED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}
