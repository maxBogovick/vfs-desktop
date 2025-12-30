/**
 * System Handlers
 */

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::api_server::{models::*, state::AppState};
use crate::api::RealFileSystem;
use crate::core::FileSystem;

#[utoipa::path(
    get,
    path = "/api/v1/system/home",
    responses(
        (status = 200, description = "Home directory path", body = HomeDirectoryResponse),
    ),
    tag = "system"
)]
pub async fn get_home_directory(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let fs = RealFileSystem::new();
    match fs.get_home_directory() {
        Ok(path) => Json(HomeDirectoryResponse { path }).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::operation_failed(err.message)),
        )
            .into_response(),
    }
}

pub async fn get_system_folders(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let fs = RealFileSystem::new();
    match fs.get_system_folders() {
        Ok(folders) => Json(SystemFoldersResponse { folders }).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::operation_failed(err.message)),
        )
            .into_response(),
    }
}

pub async fn get_system_stats(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // TODO: Implement system stats
    StatusCode::NOT_IMPLEMENTED.into_response()
}

pub async fn open_terminal(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OpenTerminalRequest>,
) -> impl IntoResponse {
    let fs = RealFileSystem::new();
    match fs.open_terminal(&req.path) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::operation_failed(err.message)),
        )
            .into_response(),
    }
}
