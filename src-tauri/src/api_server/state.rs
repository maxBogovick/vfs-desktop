/**
 * Application State
 *
 * Shared state for API server
 */

use tokio::sync::broadcast;
use std::sync::Arc;
use super::models::WebSocketMessage;

/// Application state shared across all requests
pub struct AppState {
    /// Broadcast channel for WebSocket messages
    pub ws_tx: broadcast::Sender<WebSocketMessage>,
}

impl AppState {
    pub fn new() -> Self {
        let (ws_tx, _) = broadcast::channel(100);
        Self { ws_tx }
    }

    /// Broadcast message to all WebSocket clients
    pub fn broadcast(&self, message: WebSocketMessage) {
        let _ = self.ws_tx.send(message);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
