pub mod api;
pub mod application;
pub mod core;
pub mod config;
mod error;
mod commands;
mod progress;
mod file_operations;
#[cfg(feature = "api-server")]
pub mod file_operations_async;
pub mod templates;

// Shared application state
pub mod state;

// Universal API Service Layer (business logic)
pub mod api_service;

// REST API Server (optional, for standalone mode)
#[cfg(feature = "api-server")]
pub mod api_server;

use commands::*;

// Initialize tracing on library load
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vfdir=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Filesystem commands
            read_directory,
            delete_item,
            rename_item,
            create_folder,
            create_file,
            create_files_batch,
            copy_items,
            move_items,
            get_home_directory,
            get_file_info,
            open_file,
            reveal_in_finder,
            get_system_folders,
            read_file_content,
            write_file_content,
            normalize_path,
            get_path_suggestions,
            open_terminal,
            execute_command,
            // Config commands
            get_config,
            update_config,
            set_filesystem_backend,
            // Bookmark commands
            get_bookmarks,
            add_bookmark,
            remove_bookmark,
            rename_bookmark,
            // UI State commands
            get_ui_state,
            save_ui_state,
            // Progress operations commands
            copy_items_with_progress_command,
            move_items_with_progress_command,
            delete_items_with_progress_command,
            cancel_operation,
            pause_operation,
            resume_operation,
            // Directory size calculation
            calculate_directory_size,
            // System monitoring commands
            get_system_stats,
            // Conflict resolution commands
            check_file_conflict,
            copy_file_with_custom_name,
            // Batch operations commands
            batch_change_attributes,
            validate_batch_rename,
            // Template commands
            get_file_templates,
            suggest_file_extension,
            get_template_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
