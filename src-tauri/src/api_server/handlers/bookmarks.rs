/**
 * Bookmarks Handlers
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

#[utoipa::path(
    get,
    path = "/api/v1/bookmarks",
    responses(
        (status = 200, description = "List of bookmarks", body = BookmarksResponse),
    ),
    tag = "bookmarks"
)]
pub async fn get_bookmarks(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match API.bookmarks.get_all() {
        Ok(bookmarks) => Json(BookmarksResponse { bookmarks }).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::operation_failed(err.to_string()))
        ).into_response()
    }
}

pub async fn add_bookmark(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<AddBookmarkRequest>,
) -> impl IntoResponse {
    match API.bookmarks.add(req.path, req.name) {
        Ok(bookmark) => (StatusCode::CREATED, Json(bookmark)).into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

pub async fn remove_bookmark(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match API.bookmarks.remove(&id) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

pub async fn rename_bookmark(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<RenameBookmarkRequest>,
) -> impl IntoResponse {
    match API.bookmarks.rename(&id, req.new_name) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}
