use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use local_ip_address::local_ip;
use qrcode::QrCode;
use qrcode::render::svg;
use std::sync::Mutex;
use tokio::sync::oneshot;
use once_cell::sync::Lazy;
use tracing::{info, error};
use crate::api_service::API;

// Global handle to stop the server
static SERVER_SHUTDOWN: Lazy<Mutex<Option<oneshot::Sender<()>>>> = Lazy::new(|| Mutex::new(None));

#[derive(Clone, serde::Serialize)]
pub struct ShareInfo {
    pub url: String,
    pub qr_svg: String,
    pub filename: String,
}

/// Handler to serve file content from any filesystem (Real or Virtual)
async fn download_handler(
    State((file_path, fs_backend)): State<(String, Option<String>)>,
) -> impl IntoResponse {
    // Read file content using global API (handles Real & Virtual FS)
    // Note: This reads entire file into memory. OK for Magic Share use case.
    match API.files.read_file_bytes(&file_path, fs_backend.as_deref()) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
            
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())], 
                bytes
            ).into_response()
        },
        Err(e) => {
            error!("Failed to serve file '{}': {}", file_path, e);
            axum::http::StatusCode::NOT_FOUND.into_response()
        }
    }
}

/// Starts a temporary HTTP server to share a specific file
pub async fn start_share_server(file_path: String, fs_backend: Option<String>) -> Result<ShareInfo, String> {
    // 1. Stop existing server if any
    stop_share_server();

    // 2. Verify file existence via API (abstracts FS)
    let info = API.files.get_file_info(&file_path, fs_backend.as_deref())
        .map_err(|e| format!("File not found or inaccessible: {}", e))?;
    
    let filename = info.name;

    // 3. Generate a random UUID for the route
    let share_id = uuid::Uuid::new_v4().to_string();
    
    // 4. Setup Router
    // Passes file_path and fs_backend as state to the handler
    // URL-encode filename to ensure valid path
    let encoded_filename = urlencoding::encode(&filename);
    let route_path = format!("/download/{}/{}", share_id, encoded_filename);

    let app = Router::new()
        .route(
            &route_path, 
            get(download_handler)
        )
        .with_state((file_path, fs_backend));

    // 5. Find a free port (bind to port 0)
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("Failed to bind to port: {}", e))?;
    
    let port = listener.local_addr()
        .map_err(|e| format!("Failed to get local addr: {}", e))?
        .port();

    // 6. Get LAN IP
    let my_local_ip = local_ip().map_err(|e| format!("Failed to get local IP: {}", e))?;
    
    let url = format!("http://{}:{}{}", my_local_ip, port, route_path);
    info!("Sharing file at: {}", url);

    // 7. Generate QR Code
    let code = QrCode::new(&url).map_err(|e| format!("Failed to generate QR: {}", e))?;
    let image = code.render::<svg::Color>()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    // 8. Start Server in background
    let (tx, rx) = oneshot::channel();
    
    // Store the shutdown signal
    if let Ok(mut guard) = SERVER_SHUTDOWN.lock() {
        *guard = Some(tx);
    }

    tokio::spawn(async move {
        info!("Share server started on port {}", port);
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await 
        {
            error!("Share server error: {}", e);
        }
        info!("Share server stopped");
    });

    Ok(ShareInfo {
        url,
        qr_svg: image,
        filename,
    })
}

/// Stops the shared server
pub fn stop_share_server() {
    if let Ok(mut guard) = SERVER_SHUTDOWN.lock() {
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
            info!("Stop signal sent to share server");
        }
    }
}

// Tauri Commands

#[tauri::command]
pub async fn share_file(path: String, filesystem: Option<String>) -> Result<ShareInfo, String> {
    start_share_server(path, filesystem).await
}

#[tauri::command]
pub fn stop_share() {
    stop_share_server();
}

