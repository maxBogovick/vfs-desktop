/**
 * File Operations Handlers
 */

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::api_server::{
    models::*,
    state::AppState,
};
use crate::api_service::{API, ApiError};

/// List directory contents
#[utoipa::path(
    get,
    path = "/api/v1/files",
    params(ListDirectoryQuery),
    responses(
        (status = 200, description = "Directory listing", body = ListDirectoryResponse),
        (status = 404, description = "Directory not found", body = ErrorResponse),
    ),
    tag = "files"
)]
pub async fn list_directory(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<ListDirectoryQuery>,
) -> impl IntoResponse {
    match API.files.list_directory(&query.path, query.panel_fs.as_deref()) {
        Ok(files) => Json(ListDirectoryResponse { files }).into_response(),
        Err(err) => {
            let status = match err {
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Get file information
#[utoipa::path(
    get,
    path = "/api/v1/files/info",
    params(GetFileInfoQuery),
    responses(
        (status = 200, description = "File information", body = FileSystemEntry),
        (status = 404, description = "File not found", body = ErrorResponse),
    ),
    tag = "files"
)]
pub async fn get_file_info(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<GetFileInfoQuery>,
) -> impl IntoResponse {
    match API.files.get_file_info(&query.path, query.panel_fs.as_deref()) {
        Ok(info) => Json(info).into_response(),
        Err(err) => {
            let status = match err {
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Create new folder
#[utoipa::path(
    post,
    path = "/api/v1/files/create-folder",
    request_body = CreateFolderRequest,
    responses(
        (status = 200, description = "Folder created successfully"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    ),
    tag = "files"
)]
pub async fn create_folder(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateFolderRequest>,
) -> impl IntoResponse {
    match API.files.create_folder(&req.path, &req.name, req.panel_fs.as_deref()) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                ApiError::AlreadyExists { .. } => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Copy files/folders
#[utoipa::path(
    post,
    path = "/api/v1/files/copy",
    request_body = CopyItemsRequest,
    responses(
        (status = 200, description = "Items copied successfully"),
        (status = 400, description = "Copy failed", body = ErrorResponse),
    ),
    tag = "files"
)]
/// `TODO change signature instead of panel_fs need using source_file_system, destination_file_system
pub async fn copy_items(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CopyItemsRequest>,
) -> impl IntoResponse {
    match API.files.copy_items(&req.sources, &req.destination, req.source_file_system.as_deref()) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Move files/folders
#[utoipa::path(
    post,
    path = "/api/v1/files/move",
    request_body = MoveItemsRequest,
    responses(
        (status = 200, description = "Items moved successfully"),
        (status = 400, description = "Move failed", body = ErrorResponse),
    ),
    tag = "files"
)]
/// `TODO change signature instead of panel_fs need using source_file_system, destination_file_system
pub async fn move_items(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<MoveItemsRequest>,
) -> impl IntoResponse {
    match API.files.move_items(&req.sources, &req.destination, req.source_file_system.as_deref()) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Rename file/folder
#[utoipa::path(
    post,
    path = "/api/v1/files/rename",
    request_body = RenameItemRequest,
    responses(
        (status = 200, description = "Item renamed successfully"),
        (status = 400, description = "Rename failed", body = ErrorResponse),
    ),
    tag = "files"
)]
pub async fn rename_item(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RenameItemRequest>,
) -> impl IntoResponse {
    match API.files.rename_item(&req.old_path, &req.new_name, req.panel_fs.as_deref()) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Delete files/folders
#[utoipa::path(
    delete,
    path = "/api/v1/files",
    request_body = DeleteItemsRequest,
    responses(
        (status = 200, description = "Items deleted successfully"),
        (status = 400, description = "Delete failed", body = ErrorResponse),
    ),
    tag = "files"
)]
pub async fn delete_items(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<DeleteItemsRequest>,
) -> impl IntoResponse {
    match API.files.delete_items(&req.paths, req.panel_fs.as_deref()) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Read file content
pub async fn read_file_content(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<ReadFileContentQuery>,
) -> impl IntoResponse {
    match API.files.read_file_content(&query.path, query.max_size, query.panel_fs.as_deref()) {
        Ok(content) => Json(ReadFileContentResponse { content }).into_response(),
        Err(err) => {
            let status = match err {
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Write file content
pub async fn write_file_content(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WriteFileContentRequest>,
) -> impl IntoResponse {
    match API.files.write_file_content(&req.path, &req.content, req.panel_fs.as_deref()) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Open file with system default application
pub async fn open_file(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OpenFileRequest>,
) -> impl IntoResponse {
    match API.files.open_file(&req.path, req.panel_fs.as_deref()) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Reveal file in system file manager
pub async fn reveal_in_finder(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OpenFileRequest>,
) -> impl IntoResponse {
    match API.files.reveal_in_finder(&req.path, req.panel_fs.as_deref()) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

/// Copy items with progress (WebSocket updates)
pub async fn copy_items_with_progress(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CopyItemsWithProgressRequest>,
) -> impl IntoResponse {
    use crate::file_operations::calculate_total_size;
    use crate::progress::{OPERATIONS_MANAGER, OperationType};
    use uuid::Uuid;

    let operation_id = req.operation_id.unwrap_or_else(|| Uuid::new_v4().to_string());

    // Calculate total size
    let (total_bytes, total_items) = match calculate_total_size(&req.sources) {
        Ok(size) => size,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::operation_failed(format!("Failed to calculate size: {}", e)))
            ).into_response();
        }
    };

    // Create operation tracker
    let tracker = OPERATIONS_MANAGER.create_operation(
        operation_id.clone(),
        OperationType::Copy,
        total_bytes,
        total_items,
    );

    // Spawn async task
    let state_clone = state.clone();
    let sources = req.sources.clone();
    let destination = req.destination.clone();

    tokio::spawn(async move {
        use crate::file_operations_async::copy_items_with_progress_async;
        let _ = copy_items_with_progress_async(
            sources,
            destination,
            tracker,
            state_clone,
        ).await;
    });

    Json(serde_json::json!({ "operationId": operation_id })).into_response()
}

/// Move items with progress (WebSocket updates)
pub async fn move_items_with_progress(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MoveItemsWithProgressRequest>,
) -> impl IntoResponse {
    use crate::file_operations::calculate_total_size;
    use crate::progress::{OPERATIONS_MANAGER, OperationType};
    use uuid::Uuid;

    let operation_id = req.operation_id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let (total_bytes, total_items) = match calculate_total_size(&req.sources) {
        Ok(size) => size,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::operation_failed(format!("Failed to calculate size: {}", e)))
            ).into_response();
        }
    };

    let tracker = OPERATIONS_MANAGER.create_operation(
        operation_id.clone(),
        OperationType::Move,
        total_bytes,
        total_items,
    );

    let state_clone = state.clone();
    let sources = req.sources.clone();
    let destination = req.destination.clone();

    tokio::spawn(async move {
        use crate::file_operations_async::move_items_with_progress_async;
        let _ = move_items_with_progress_async(
            sources,
            destination,
            tracker,
            state_clone,
        ).await;
    });

    Json(serde_json::json!({ "operationId": operation_id })).into_response()
}

/// Delete items with progress (WebSocket updates)
pub async fn delete_items_with_progress(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteItemsWithProgressRequest>,
) -> impl IntoResponse {
    use crate::file_operations::calculate_total_size;
    use crate::progress::{OPERATIONS_MANAGER, OperationType};
    use uuid::Uuid;

    let operation_id = req.operation_id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let (total_bytes, total_items) = match calculate_total_size(&req.paths) {
        Ok(size) => size,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::operation_failed(format!("Failed to calculate size: {}", e)))
            ).into_response();
        }
    };

    let tracker = OPERATIONS_MANAGER.create_operation(
        operation_id.clone(),
        OperationType::Delete,
        total_bytes,
        total_items,
    );

    let state_clone = state.clone();
    let paths = req.paths.clone();

    tokio::spawn(async move {
        use crate::file_operations_async::delete_items_with_progress_async;
        let _ = delete_items_with_progress_async(
            paths,
            tracker,
            state_clone,
        ).await;
    });

    Json(serde_json::json!({ "operationId": operation_id })).into_response()
}
