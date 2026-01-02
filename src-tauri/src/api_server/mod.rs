/**
 * REST API Server Module
 *
 * Provides HTTP REST API for file manager operations
 * that can be used by web, mobile, and console clients.
 */

pub mod routes;
pub mod handlers;
pub mod models;
pub mod websocket;
pub mod state;

use axum::{
    Router,
    routing::{get, post, put, delete},
};
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub use state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::files::list_directory,
        handlers::files::get_file_info,
        handlers::files::create_folder,
        handlers::files::copy_items,
        handlers::files::move_items,
        handlers::files::rename_item,
        handlers::files::delete_items,
        handlers::bookmarks::get_bookmarks,
        handlers::system::get_home_directory,
    ),
    components(
        schemas(
            models::FileSystemEntry,
            models::ErrorResponse,
            models::CreateFolderRequest,
            models::CopyItemsRequest,
            models::MoveItemsRequest,
            models::RenameItemRequest,
            models::DeleteItemsRequest,
        )
    ),
    tags(
        (name = "files", description = "File system operations"),
        (name = "batch", description = "Batch operations"),
        (name = "bookmarks", description = "Bookmark management"),
        (name = "system", description = "System operations"),
        (name = "config", description = "Configuration"),
    )
)]
struct ApiDoc;

/// Create API router with all endpoints
pub fn create_router(state: Arc<AppState>) -> Router {
    // API routes
    let api_routes = Router::new()
        // File system operations
        .route("/files", get(handlers::files::list_directory))
        .route("/files/info", get(handlers::files::get_file_info))
        .route("/files/create-folder", post(handlers::files::create_folder))
        .route("/files/copy", post(handlers::files::copy_items))
        .route("/files/move", post(handlers::files::move_items))
        .route("/files/rename", post(handlers::files::rename_item))
        .route("/files", delete(handlers::files::delete_items))
        .route("/files/content", get(handlers::files::read_file_content))
        .route("/files/content", post(handlers::files::write_file_content))
        .route("/files/open", post(handlers::files::open_file))
        .route("/files/reveal", post(handlers::files::reveal_in_finder))

        // File operations with progress (WebSocket updates)
        .route("/files/copy-with-progress", post(handlers::files::copy_items_with_progress))
        .route("/files/move-with-progress", post(handlers::files::move_items_with_progress))
        .route("/files/delete-with-progress", post(handlers::files::delete_items_with_progress))

        // Batch operations
        .route("/batch/rename", post(handlers::batch::batch_rename))
        .route("/batch/rename/preview", post(handlers::batch::batch_rename_preview))
        .route("/batch/attributes", post(handlers::batch::batch_attributes))
        .route("/batch/operations", get(handlers::batch::get_operations))
        .route("/batch/operations/:id", get(handlers::batch::get_operation))
        .route("/batch/operations/:id", delete(handlers::batch::cancel_operation))
        .route("/batch/operations/:id/retry", post(handlers::batch::retry_operation))

        // Bookmarks
        .route("/bookmarks", get(handlers::bookmarks::get_bookmarks))
        .route("/bookmarks", post(handlers::bookmarks::add_bookmark))
        .route("/bookmarks/:id", delete(handlers::bookmarks::remove_bookmark))
        .route("/bookmarks/:id", put(handlers::bookmarks::rename_bookmark))

        // System
        .route("/system/home", get(handlers::system::get_home_directory))
        .route("/system/folders", get(handlers::system::get_system_folders))
        .route("/system/stats", get(handlers::system::get_system_stats))
        .route("/system/terminal", post(handlers::system::open_terminal))

        // Config
        .route("/config", get(handlers::config::get_config))
        .route("/config", put(handlers::config::update_config))
        .route("/config/ui-state", get(handlers::config::get_ui_state))
        .route("/config/ui-state", put(handlers::config::save_ui_state))

        // WebSocket
        .route("/ws/operations", get(websocket::operations_websocket_handler))
        .route("/ws/filesystem", get(websocket::filesystem_websocket_handler))

        .with_state(state);

    // Main router with API version prefix
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_routes)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any))
        )
}

/// Start the API server
pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new());
    let app = create_router(state);

    println!("🚀 VFDir API Server starting on http://{}", addr);
    println!("📚 API Documentation: http://{}/swagger-ui/", addr);
    println!("🔌 WebSocket Operations: ws://{}/api/v1/ws/operations", addr);
    println!("📁 WebSocket FileSystem: ws://{}/api/v1/ws/filesystem", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
