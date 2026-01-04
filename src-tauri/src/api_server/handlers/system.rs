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
use crate::api_service::{API, ApiError};

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
    Json(req): Json<GetHomeDirRequest>,
) -> impl IntoResponse {
    match API.system.get_home_directory(req.panel_fs.as_deref()) {
        Ok(path) => Json(HomeDirectoryResponse { path }).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::operation_failed(err.to_string())),
        ).into_response(),
    }
}

pub async fn get_system_folders(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<GetHomeDirRequest>,
) -> impl IntoResponse {
    match API.system.get_system_folders(req.panel_fs.as_deref()) {
        Ok(folders) => Json(SystemFoldersResponse { folders }).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::operation_failed(err.to_string())),
        ).into_response(),
    }
}

pub async fn get_system_stats(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match API.system.get_stats() {
        Ok(stats) => Json(stats).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::operation_failed(err.to_string())),
        ).into_response(),
    }
}

pub async fn open_terminal(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OpenTerminalRequest>,
) -> impl IntoResponse {
    match API.system.open_terminal(&req.path) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::FileNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}
