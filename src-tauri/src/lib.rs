pub mod api;
pub mod application;
pub mod core;
pub mod config;
mod error;
mod commands;
mod archives;
mod progress;
mod file_operations;
#[cfg(feature = "api-server")]
pub mod file_operations_async;
pub mod templates;
pub mod queue;

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
    init_tracing();

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
            get_template_content,
            // Archive commands
            extract_archive,
            list_archive_contents,
            create_archive,
            // Vault security commands
            vault_is_enabled,
            vault_get_status,
            vault_initialize,
            vault_unlock,
            vault_lock,
            // Vault recovery commands
            vault_setup_recovery,
            vault_request_password_reset,
            vault_verify_reset_code,
            vault_get_recovery_channels,
            vault_is_recovery_configured,
            vault_reset,
            // Vault directory management commands
            vault_get_current_directory,
            vault_get_default_directory,
            vault_select_directory,
            vault_set_custom_directory,
            vault_reset_to_default_directory,
            // Queue management commands
            queue_add_operation,
            queue_get_all_operations,
            queue_get_operation,
            queue_cancel_operation,
            queue_retry_operation,
            queue_remove_operation,
            queue_update_config,
            queue_get_config,
            queue_pause_operation,
            queue_resume_operation,
            queue_run_now,
            // Share commands
            api::share::share_file,
            api::share::stop_share,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
