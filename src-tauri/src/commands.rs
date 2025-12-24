use crate::api::RealFileSystem;
use crate::api::virtual_fs::VirtualFileSystem;
use crate::config::{AppConfig, FileSystemBackend};
use crate::core::{FileSystem, FileSystemEntry};
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;

// Глобальное состояние конфигурации
static APP_CONFIG: Lazy<Arc<RwLock<AppConfig>>> = Lazy::new(|| {
    let config = AppConfig::load().unwrap_or_default();
    Arc::new(RwLock::new(config))
});

// Enum для хранения разных типов файловых систем
enum FileSystemInstance {
    Real(RealFileSystem),
    Virtual(VirtualFileSystem),
}

impl FileSystemInstance {
    fn as_trait(&self) -> &dyn FileSystem {
        match self {
            FileSystemInstance::Real(fs) => fs,
            FileSystemInstance::Virtual(fs) => fs,
        }
    }
}

// Получить экземпляр файловой системы на основе конфига
fn get_filesystem() -> FileSystemInstance {
    let config = APP_CONFIG.read().unwrap();

    match config.filesystem_backend {
        FileSystemBackend::Real => {
            FileSystemInstance::Real(RealFileSystem::new())
        }
        FileSystemBackend::Virtual => {
            let virtual_fs = VirtualFileSystem::new("/Users/maxim/Projects/Rust/vfdir/out/fs.json")
                .unwrap_or_else(|_| VirtualFileSystem::new("/tmp/vfdir_fs.json").unwrap());
            FileSystemInstance::Virtual(virtual_fs)
        }
    }
}

// ====== Команды для работы с конфигурацией ======

#[tauri::command]
pub fn get_config() -> Result<AppConfig, String> {
    let config = APP_CONFIG.read().unwrap();
    Ok(config.clone())
}

#[tauri::command]
pub fn update_config(new_config: AppConfig) -> Result<(), String> {
    // Сохранить в файл
    new_config.save()?;

    // Обновить в памяти
    let mut config = APP_CONFIG.write().unwrap();
    *config = new_config;

    Ok(())
}

#[tauri::command]
pub fn set_filesystem_backend(backend: String) -> Result<(), String> {
    let backend_enum = match backend.as_str() {
        "real" => FileSystemBackend::Real,
        "virtual" => FileSystemBackend::Virtual,
        _ => return Err("Invalid backend type. Use 'real' or 'virtual'".to_string()),
    };

    let mut config = APP_CONFIG.write().unwrap();
    config.filesystem_backend = backend_enum;
    config.save()?;

    Ok(())
}

#[tauri::command]
pub fn read_directory(path: String) -> Result<Vec<FileSystemEntry>, String> {
    let fs = get_filesystem();
    fs.as_trait().read_directory(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn delete_item(path: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait().delete_item(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn rename_item(old_path: String, new_name: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait().rename_item(&old_path, &new_name).map_err(|e| e.message)
}

#[tauri::command]
pub fn create_folder(path: String, name: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait().create_folder(&path, &name).map_err(|e| e.message)
}

#[tauri::command]
pub fn copy_items(sources: Vec<String>, destination: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait().copy_items(&sources, &destination).map_err(|e| e.message)
}

#[tauri::command]
pub fn move_items(sources: Vec<String>, destination: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait().move_items(&sources, &destination).map_err(|e| e.message)
}

#[tauri::command]
pub fn get_home_directory() -> Result<String, String> {
    let fs = get_filesystem();
    fs.as_trait().get_home_directory().map_err(|e| e.message)
}

#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileSystemEntry, String> {
    let fs = get_filesystem();
    fs.as_trait().get_file_info(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait().open_file(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn reveal_in_finder(path: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait().reveal_in_finder(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn get_system_folders() -> Result<Vec<FileSystemEntry>, String> {
    let fs = get_filesystem();
    fs.as_trait().get_system_folders().map_err(|e| e.message)
}

#[tauri::command]
pub fn read_file_content(path: String, max_size: Option<u64>) -> Result<String, String> {
    let fs = get_filesystem();
    fs.as_trait().read_file_content(&path, max_size).map_err(|e| e.message)
}

#[tauri::command]
pub fn normalize_path(path: String) -> Result<String, String> {
    let fs = get_filesystem();
    fs.as_trait().normalize_path(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn get_path_suggestions(partial_path: String) -> Result<Vec<String>, String> {
    let fs = get_filesystem();
    fs.as_trait().get_path_suggestions(&partial_path).map_err(|e| e.message)
}

#[tauri::command]
pub fn open_terminal(path: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait().open_terminal(&path).map_err(|e| e.message)
}
