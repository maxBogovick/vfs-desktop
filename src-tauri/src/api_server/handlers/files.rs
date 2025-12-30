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
use crate::api::RealFileSystem;
use crate::core::FileSystem;

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
    let fs = RealFileSystem::new();

    match fs.read_directory(&query.path) {
        Ok(files) => Json(ListDirectoryResponse { files }).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::file_not_found(&query.path)),
        )
            .into_response(),
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
    let fs = RealFileSystem::new();

    match fs.get_file_info(&query.path) {
        Ok(info) => Json(info).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::file_not_found(&query.path)),
        )
            .into_response(),
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
    let fs = RealFileSystem::new();

    match fs.create_folder(&req.path, &req.name) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::operation_failed(err.message)),
        )
            .into_response(),
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
pub async fn copy_items(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CopyItemsRequest>,
) -> impl IntoResponse {
    let fs = RealFileSystem::new();

    match fs.copy_items(&req.sources, &req.destination) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::operation_failed(err.message)),
        )
            .into_response(),
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
pub async fn move_items(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<MoveItemsRequest>,
) -> impl IntoResponse {
    let fs = RealFileSystem::new();

    match fs.move_items(&req.sources, &req.destination) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::operation_failed(err.message)),
        )
            .into_response(),
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
    let fs = RealFileSystem::new();

    match fs.rename_item(&req.old_path, &req.new_name) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::operation_failed(err.message)),
        )
            .into_response(),
    }
}

/// Delete files/folders
pub async fn delete_items(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<DeleteItemsRequest>,
) -> impl IntoResponse {
    let fs = RealFileSystem::new();

    for path in &req.paths {
        if let Err(err) = fs.delete_item(path) {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::operation_failed(err.message)),
            )
                .into_response();
        }
    }

    StatusCode::OK.into_response()
}

/// Read file content
pub async fn read_file_content(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<ReadFileContentQuery>,
) -> impl IntoResponse {
    let fs = RealFileSystem::new();

    match fs.read_file_content(&query.path, query.max_size) {
        Ok(content) => Json(ReadFileContentResponse { content }).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::file_not_found(&query.path)),
        )
            .into_response(),
    }
}

/// Open file with system default application
pub async fn open_file(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OpenFileRequest>,
) -> impl IntoResponse {
    let fs = RealFileSystem::new();

    match fs.open_file(&req.path) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::operation_failed(err.message)),
        )
            .into_response(),
    }
}

/// Reveal file in system file manager
pub async fn reveal_in_finder(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OpenFileRequest>,
) -> impl IntoResponse {
    let fs = RealFileSystem::new();

    match fs.reveal_in_finder(&req.path) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::operation_failed(err.message)),
        )
            .into_response(),
    }
}
