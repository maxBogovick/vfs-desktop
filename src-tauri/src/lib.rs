pub mod api;
pub mod application;
pub mod core;
mod error;
mod commands;

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
            get_path_suggestions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
