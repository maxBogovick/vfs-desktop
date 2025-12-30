/**
 * Configuration Handlers
 */

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::api_server::{models::*, state::AppState};
use crate::config::APP_CONFIG;

pub async fn get_config(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let config = APP_CONFIG.read().unwrap();
    Json(config.clone()).into_response()
}

pub async fn update_config(
    State(_state): State<Arc<AppState>>,
    Json(new_config): Json<AppConfig>,
) -> impl IntoResponse {
    // TODO: Implement update config
    StatusCode::NOT_IMPLEMENTED.into_response()
}

pub async fn get_ui_state(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let config = APP_CONFIG.read().unwrap();
    Json(&config.ui_state).into_response()
}

pub async fn save_ui_state(
    State(_state): State<Arc<AppState>>,
    Json(ui_state): Json<UIState>,
) -> impl IntoResponse {
    // TODO: Implement save UI state
    StatusCode::NOT_IMPLEMENTED.into_response()
}
