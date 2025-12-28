pub mod api;
pub mod application;
pub mod core;
pub mod config;
mod error;
mod commands;
mod progress;
mod file_operations;

use commands::*;

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
            copy_items,
            move_items,
            get_home_directory,
            get_file_info,
            open_file,
            reveal_in_finder,
            get_system_folders,
            read_file_content,
            normalize_path,
            get_path_suggestions,
            open_terminal,
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
            get_system_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
