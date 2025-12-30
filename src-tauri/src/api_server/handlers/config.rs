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
use crate::api_service::{API, ApiError};

pub async fn get_config(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match API.config.get() {
        Ok(config) => Json(config).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::operation_failed(err.to_string()))
        ).into_response()
    }
}

pub async fn update_config(
    State(_state): State<Arc<AppState>>,
    Json(new_config): Json<AppConfig>,
) -> impl IntoResponse {
    match API.config.update(new_config) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            let status = match err {
                ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse::operation_failed(err.to_string()))).into_response()
        }
    }
}

pub async fn get_ui_state(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match API.config.get_ui_state() {
        Ok(ui_state) => Json(ui_state).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::operation_failed(err.to_string()))
        ).into_response()
    }
}

pub async fn save_ui_state(
    State(_state): State<Arc<AppState>>,
    Json(ui_state): Json<UIState>,
) -> impl IntoResponse {
    match API.config.save_ui_state(ui_state) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::operation_failed(err.to_string()))
        ).into_response()
    }
}
