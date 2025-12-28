use crate::api::virtual_fs::VirtualFileSystem;
use crate::api::RealFileSystem;
use crate::config::{AppConfig, Bookmark, FileSystemBackend, UIState};
use crate::core::{FileSystem, FileSystemEntry};
use crate::progress::{emit_progress, OperationType, OPERATIONS_MANAGER};
use crate::file_operations::{
    calculate_total_size, copy_items_with_progress, delete_items_with_progress,
    move_items_with_progress,
};
use once_cell::sync::Lazy;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Runtime};
use sysinfo::{System, Pid};
use serde::Serialize;

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
        FileSystemBackend::Real => FileSystemInstance::Real(RealFileSystem::new()),
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
    fs.as_trait()
        .rename_item(&old_path, &new_name)
        .map_err(|e| e.message)
}

#[tauri::command]
pub fn create_folder(path: String, name: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait()
        .create_folder(&path, &name)
        .map_err(|e| e.message)
}

#[tauri::command]
pub fn copy_items(sources: Vec<String>, destination: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait()
        .copy_items(&sources, &destination)
        .map_err(|e| e.message)
}

#[tauri::command]
pub fn move_items(sources: Vec<String>, destination: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait()
        .move_items(&sources, &destination)
        .map_err(|e| e.message)
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
    fs.as_trait()
        .read_file_content(&path, max_size)
        .map_err(|e| e.message)
}

#[tauri::command]
pub fn normalize_path(path: String) -> Result<String, String> {
    let fs = get_filesystem();
    fs.as_trait().normalize_path(&path).map_err(|e| e.message)
}

#[tauri::command]
pub fn get_path_suggestions(partial_path: String) -> Result<Vec<String>, String> {
    let fs = get_filesystem();
    fs.as_trait()
        .get_path_suggestions(&partial_path)
        .map_err(|e| e.message)
}

#[tauri::command]
pub fn open_terminal(path: String) -> Result<(), String> {
    let fs = get_filesystem();
    fs.as_trait().open_terminal(&path).map_err(|e| e.message)
}

// ====== Команды для работы с закладками ======

#[tauri::command]
pub fn get_bookmarks() -> Result<Vec<Bookmark>, String> {
    let config = APP_CONFIG.read().unwrap();
    Ok(config.bookmarks.clone())
}

#[tauri::command]
pub fn add_bookmark(path: String, name: Option<String>) -> Result<Bookmark, String> {
    let mut config = APP_CONFIG.write().unwrap();

    // Проверить, не существует ли уже закладка с таким путем
    if config.bookmarks.iter().any(|b| b.path == path) {
        return Err("Bookmark with this path already exists".to_string());
    }

    // Создать имя из пути, если не указано
    let bookmark_name = name.unwrap_or_else(|| {
        std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Bookmark")
            .to_string()
    });

    // Получить текущее время
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Создать уникальный ID
    let id = format!("bookmark_{}", timestamp);

    let bookmark = Bookmark {
        id: id.clone(),
        name: bookmark_name,
        path: path.clone(),
        created_at: timestamp,
    };

    config.bookmarks.push(bookmark.clone());

    // Сохранить конфигурацию
    config.save()?;

    Ok(bookmark)
}

#[tauri::command]
pub fn remove_bookmark(id: String) -> Result<(), String> {
    let mut config = APP_CONFIG.write().unwrap();

    let initial_len = config.bookmarks.len();
    config.bookmarks.retain(|b| b.id != id);

    if config.bookmarks.len() == initial_len {
        return Err("Bookmark not found".to_string());
    }

    config.save()?;
    Ok(())
}

#[tauri::command]
pub fn rename_bookmark(id: String, new_name: String) -> Result<(), String> {
    let mut config = APP_CONFIG.write().unwrap();

    let bookmark = config
        .bookmarks
        .iter_mut()
        .find(|b| b.id == id)
        .ok_or_else(|| "Bookmark not found".to_string())?;

    bookmark.name = new_name;

    config.save()?;
    Ok(())
}

// ====== Команды для работы с UI состоянием ======

#[tauri::command]
pub fn get_ui_state() -> Result<UIState, String> {
    let config = APP_CONFIG.read().unwrap();
    let ui_state = config.ui_state.clone();

    println!("[get_ui_state] Returning UI state:");
    println!("  - tabs: {}", ui_state.tabs.len());
    println!("  - active_tab_id: {:?}", ui_state.active_tab_id);
    println!("  - last_path: {:?}", ui_state.last_path);
    println!("  - sidebar_width: {}", ui_state.sidebar_width);
    println!("  - sidebar expanded_folders: {}", ui_state.sidebar.expanded_folders.len());
    println!("  - sidebar quick_access: {}", ui_state.sidebar.quick_access_expanded);
    println!("  - sidebar folder_tree: {}", ui_state.sidebar.folder_tree_expanded);
    println!("  - sidebar favorites: {}", ui_state.sidebar.favorites_expanded);

    Ok(ui_state)
}

#[tauri::command]
pub fn save_ui_state(ui_state: UIState) -> Result<(), String> {
    println!("[save_ui_state] Received UI state:");
    println!("  - tabs: {}", ui_state.tabs.len());
    println!("  - active_tab_id: {:?}", ui_state.active_tab_id);
    println!("  - last_path: {:?}", ui_state.last_path);
    println!("  - sidebar_width: {}", ui_state.sidebar_width);
    println!("  - sidebar expanded_folders: {}", ui_state.sidebar.expanded_folders.len());
    println!("  - sidebar quick_access: {}", ui_state.sidebar.quick_access_expanded);
    println!("  - sidebar folder_tree: {}", ui_state.sidebar.folder_tree_expanded);
    println!("  - sidebar favorites: {}", ui_state.sidebar.favorites_expanded);

    let mut config = APP_CONFIG.write().unwrap();
    config.ui_state = ui_state;

    let config_path = crate::config::AppConfig::config_path()?;
    println!("[save_ui_state] Saving to file: {:?}", config_path);
    config.save()?;
    println!("[save_ui_state] ✅ Saved successfully to {:?}", config_path);

    Ok(())
}

// ====== Команды для операций с прогрессом ======

/// Копирование файлов с прогрессом
#[tauri::command]
pub async fn copy_items_with_progress_command<R: Runtime>(
    app: AppHandle<R>,
    operation_id: String,
    sources: Vec<String>,
    destination: String,
) -> Result<(), String> {
    // Вычисляем общий размер
    let (total_bytes, total_items) = calculate_total_size(&sources)
        .map_err(|e| format!("Failed to calculate size: {}", e))?;

    // Создаем трекер прогресса
    let tracker = OPERATIONS_MANAGER.create_operation(
        operation_id.clone(),
        OperationType::Copy,
        total_bytes,
        total_items,
    );

    // Отправляем начальное событие
    emit_progress(&app, &tracker);

    // Выполняем операцию
    match copy_items_with_progress(&sources, &destination, &tracker, &app) {
        Ok(_) => {
            // Убеждаемся, что прогресс на 100%
            tracker.set_total_bytes(tracker.get_current_bytes());
            tracker.mark_completed();
            // Принудительно отправляем финальное событие
            let event = tracker.get_progress_event();
            let _ = app.emit("file-operation-progress", event);
            Ok(())
        }
        Err(e) => {
            tracker.mark_failed(e.message.clone());
            emit_progress(&app, &tracker);
            Err(e.message)
        }
    }
}

/// Перемещение файлов с прогрессом
#[tauri::command]
pub async fn move_items_with_progress_command<R: Runtime>(
    app: AppHandle<R>,
    operation_id: String,
    sources: Vec<String>,
    destination: String,
) -> Result<(), String> {
    // Вычисляем общий размер
    let (total_bytes, total_items) = calculate_total_size(&sources)
        .map_err(|e| format!("Failed to calculate size: {}", e))?;

    // Создаем трекер прогресса
    let tracker = OPERATIONS_MANAGER.create_operation(
        operation_id.clone(),
        OperationType::Move,
        total_bytes,
        total_items,
    );

    // Отправляем начальное событие
    emit_progress(&app, &tracker);

    // Выполняем операцию
    match move_items_with_progress(&sources, &destination, &tracker, &app) {
        Ok(_) => {
            // Убеждаемся, что прогресс на 100%
            tracker.set_total_bytes(tracker.get_current_bytes());
            tracker.mark_completed();
            // Принудительно отправляем финальное событие
            let event = tracker.get_progress_event();
            let _ = app.emit("file-operation-progress", event);
            Ok(())
        }
        Err(e) => {
            tracker.mark_failed(e.message.clone());
            emit_progress(&app, &tracker);
            Err(e.message)
        }
    }
}

/// Удаление файлов с прогрессом
#[tauri::command]
pub async fn delete_items_with_progress_command<R: Runtime>(
    app: AppHandle<R>,
    operation_id: String,
    paths: Vec<String>,
) -> Result<(), String> {
    // Вычисляем общий размер
    let (total_bytes, total_items) = calculate_total_size(&paths)
        .map_err(|e| format!("Failed to calculate size: {}", e))?;

    // Создаем трекер прогресса
    let tracker = OPERATIONS_MANAGER.create_operation(
        operation_id.clone(),
        OperationType::Delete,
        total_bytes,
        total_items,
    );

    // Отправляем начальное событие
    emit_progress(&app, &tracker);

    // Выполняем операцию
    match delete_items_with_progress(&paths, &tracker, &app) {
        Ok(_) => {
            // Убеждаемся, что прогресс на 100%
            tracker.set_total_bytes(tracker.get_current_bytes());
            tracker.mark_completed();
            // Принудительно отправляем финальное событие
            let event = tracker.get_progress_event();
            let _ = app.emit("file-operation-progress", event);
            Ok(())
        }
        Err(e) => {
            tracker.mark_failed(e.message.clone());
            emit_progress(&app, &tracker);
            Err(e.message)
        }
    }
}

/// Отмена операции
#[tauri::command]
pub fn cancel_operation(operation_id: String) -> Result<(), String> {
    if OPERATIONS_MANAGER.cancel_operation(&operation_id) {
        Ok(())
    } else {
        Err(format!("Operation not found: {}", operation_id))
    }
}

/// Пауза операции
#[tauri::command]
pub fn pause_operation(operation_id: String) -> Result<(), String> {
    if OPERATIONS_MANAGER.pause_operation(&operation_id) {
        Ok(())
    } else {
        Err(format!("Operation not found: {}", operation_id))
    }
}

/// Возобновление операции
#[tauri::command]
pub fn resume_operation(operation_id: String) -> Result<(), String> {
    if OPERATIONS_MANAGER.resume_operation(&operation_id) {
        Ok(())
    } else {
        Err(format!("Operation not found: {}", operation_id))
    }
}

// ====== Команды для мониторинга системы ======

#[derive(Serialize)]
pub struct SystemStats {
    memory_mb: f64,
    cpu_percent: f32,
}

/// Получить статистику потребления ресурсов приложением
#[tauri::command]
pub fn get_system_stats() -> Result<SystemStats, String> {
    let mut sys = System::new_all();

    // Получаем PID текущего процесса
    let pid = Pid::from_u32(std::process::id());

    // Обновляем данные
    sys.refresh_all();

    // Получаем информацию о процессе
    if let Some(process) = sys.process(pid) {
        let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
        let cpu_percent = process.cpu_usage();

        Ok(SystemStats {
            memory_mb,
            cpu_percent,
        })
    } else {
        Err("Failed to get process information".to_string())
    }
}
