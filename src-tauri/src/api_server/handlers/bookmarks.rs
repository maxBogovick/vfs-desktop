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
use crate::config::APP_CONFIG;

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
    let config = APP_CONFIG.read().unwrap();
    Json(BookmarksResponse {
        bookmarks: config.bookmarks.clone(),
    })
    .into_response()
}

pub async fn add_bookmark(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<AddBookmarkRequest>,
) -> impl IntoResponse {
    // TODO: Implement add bookmark
    StatusCode::NOT_IMPLEMENTED.into_response()
}

pub async fn remove_bookmark(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // TODO: Implement remove bookmark
    StatusCode::NOT_IMPLEMENTED.into_response()
}

pub async fn rename_bookmark(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<RenameBookmarkRequest>,
) -> impl IntoResponse {
    // TODO: Implement rename bookmark
    StatusCode::NOT_IMPLEMENTED.into_response()
}
