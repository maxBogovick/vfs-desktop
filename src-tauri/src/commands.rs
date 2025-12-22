use crate::api::RealFileSystem;
use crate::core::{FileSystem, FileSystemEntry};

// Глобальный экземпляр файловой системы
// Можно заменить на другую реализацию (VirtualFS, DbFS) без изменения команд
fn get_filesystem() -> impl FileSystem {
    RealFileSystem::new()
}

#[tauri::command]
pub fn read_directory(path: String) -> Result<Vec<FileSystemEntry>, String> {
    let fs = get_filesystem();
    fs.read_directory(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn delete_item(path: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.delete_item(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn rename_item(old_path: String, new_name: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.rename_item(&old_path, &new_name).map_err(|e| e.message)
}

#[tauri::command]
pub fn create_folder(path: String, name: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.create_folder(&path, &name).map_err(|e| e.message)
}

#[tauri::command]
pub fn copy_items(sources: Vec<String>, destination: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.copy_items(&sources, &destination).map_err(|e| e.message)
}

#[tauri::command]
pub fn move_items(sources: Vec<String>, destination: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.move_items(&sources, &destination).map_err(|e| e.message)
}

#[tauri::command]
pub fn get_home_directory() -> Result<String, String> {
    let fs = get_filesystem();
    fs.get_home_directory().map_err(|e| e.message)
}

#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileSystemEntry, String> {
    let fs = get_filesystem();
    fs.get_file_info(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.open_file(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn reveal_in_finder(path: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.reveal_in_finder(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn get_system_folders() -> Result<Vec<FileSystemEntry>, String> {
    let fs = get_filesystem();
    fs.get_system_folders().map_err(|e| e.message)
}

#[tauri::command]
pub fn read_file_content(path: String, max_size: Option<u64>) -> Result<String, String> {
    let fs = get_filesystem();
    fs.read_file_content(&path, max_size).map_err(|e| e.message)
}
