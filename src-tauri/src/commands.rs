use crate::api_service::API;
use crate::config::{AppConfig, Bookmark, UIState};
use crate::core::FileSystemEntry;
use crate::progress::{emit_progress, OperationType, OPERATIONS_MANAGER};
use crate::file_operations::{
    calculate_total_size, copy_items_with_progress, delete_items_with_progress,
    move_items_with_progress,
};
use tauri::{AppHandle, Emitter, Runtime};
use serde::Serialize;

// ====== Команды для работы с конфигурацией ======

#[tauri::command]
pub fn get_config() -> Result<AppConfig, String> {
    API.config.get().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_config(new_config: AppConfig) -> Result<(), String> {
    API.config.update(new_config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_filesystem_backend(backend: String) -> Result<(), String> {
    API.config.set_filesystem_backend(&backend).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_directory(path: String, panel_fs: Option<String>) -> Result<Vec<FileSystemEntry>, String> {
    API.files.list_directory(&path, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_item(path: String, panel_fs: Option<String>) -> Result<(), String> {
    API.files.delete_item(&path, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_item(old_path: String, new_name: String, panel_fs: Option<String>) -> Result<(), String> {
    API.files.rename_item(&old_path, &new_name, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_folder(path: String, name: String, panel_fs: Option<String>) -> Result<(), String> {
    API.files.create_folder(&path, &name, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_file(path: String, name: String, content: Option<String>, panel_fs: Option<String>) -> Result<(), String> {
    API.files.create_file(&path, &name, content.as_deref(), panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSpec {
    pub name: String,
    pub content: Option<String>,
}

#[tauri::command]
pub fn create_files_batch(path: String, files: Vec<FileSpec>, panel_fs: Option<String>) -> Result<crate::api_service::files::BatchCreateResult, String> {
    let file_specs: Vec<(String, Option<String>)> = files
        .into_iter()
        .map(|f| (f.name, f.content))
        .collect();

    API.files.create_files_batch(&path, &file_specs, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_templates() -> Result<Vec<crate::templates::FileTemplate>, String> {
    let registry = crate::templates::TemplateRegistry::new();
    Ok(registry.get_all_templates())
}

#[tauri::command]
pub fn suggest_file_extension(path: String, panel_fs: Option<String>) -> Result<Option<String>, String> {
    // Получаем список файлов в директории
    let entries = API.files.list_directory(&path, panel_fs.as_deref()).map_err(|e| e.to_string())?;

    let file_names: Vec<String> = entries
        .into_iter()
        .filter(|e| !e.is_dir)
        .map(|e| e.name)
        .collect();

    let registry = crate::templates::TemplateRegistry::new();
    Ok(registry.suggest_extension(&file_names))
}

#[tauri::command]
pub fn get_template_content(template_id: String) -> Result<String, String> {
    let registry = crate::templates::TemplateRegistry::new();
    registry
        .get_template_by_id(&template_id)
        .map(|t| t.content)
        .ok_or_else(|| format!("Template not found: {}", template_id))
}

#[tauri::command]
pub fn copy_items(sources: Vec<String>, destination: String, panel_fs: Option<String>) -> Result<(), String> {
    API.files.copy_items(&sources, &destination, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_items(sources: Vec<String>, destination: String, panel_fs: Option<String>) -> Result<(), String> {
    API.files.move_items(&sources, &destination, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_home_directory(panel_fs: Option<String>) -> Result<String, String> {
    API.system.get_home_directory(panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_info(path: String, panel_fs: Option<String>) -> Result<FileSystemEntry, String> {
    API.files.get_file_info(&path, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_file(path: String, panel_fs: Option<String>) -> Result<(), String> {
    API.files.open_file(&path, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reveal_in_finder(path: String, panel_fs: Option<String>) -> Result<(), String> {
    API.files.reveal_in_finder(&path, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_system_folders(panel_fs: Option<String>) -> Result<Vec<FileSystemEntry>, String> {
    API.system.get_system_folders(panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_file_content(path: String, max_size: Option<u64>, panel_fs: Option<String>) -> Result<String, String> {
    API.files.read_file_content(&path, max_size, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_file_content(path: String, content: String, panel_fs: Option<String>) -> Result<(), String> {
    API.files.write_file_content(&path, &content, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn normalize_path(path: String, panel_fs: Option<String>) -> Result<String, String> {
    API.files.normalize_path(&path, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_path_suggestions(partial_path: String, panel_fs: Option<String>) -> Result<Vec<String>, String> {
    API.files.get_path_suggestions(&partial_path, panel_fs.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_terminal(path: String) -> Result<(), String> {
    API.system.open_terminal(&path).map_err(|e| e.to_string())
}

// ====== Команды для работы с терминалом ======

use crate::api_service::models::CommandResult;

#[tauri::command]
pub fn execute_command(
    command: String,
    working_dir: String,
) -> Result<CommandResult, String> {
    API.system.execute_shell_command(&command, &working_dir)
        .map_err(|e| e.to_string())
}

// ====== Команды для работы с закладками ======

#[tauri::command]
pub fn get_bookmarks() -> Result<Vec<Bookmark>, String> {
    API.bookmarks.get_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_bookmark(path: String, name: Option<String>) -> Result<Bookmark, String> {
    API.bookmarks.add(path, name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_bookmark(id: String) -> Result<(), String> {
    API.bookmarks.remove(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_bookmark(id: String, new_name: String) -> Result<(), String> {
    API.bookmarks.rename(&id, new_name).map_err(|e| e.to_string())
}

// ====== Archive Commands ======

#[tauri::command]
pub fn extract_archive(archive_path: String, destination_path: String) -> Result<(), String> {
    crate::archives::extract_archive(&archive_path, &destination_path)
}

#[tauri::command]
pub fn list_archive_contents(archive_path: String, panel_fs: Option<String>) -> Result<Vec<FileSystemEntry>, String> {
    crate::archives::list_archive_contents_with_fs(&archive_path, panel_fs.as_deref())
}

#[tauri::command]
pub fn create_archive(source_paths: Vec<String>, destination_path: String) -> Result<(), String> {
    crate::archives::create_archive(source_paths, destination_path)
}

// ====== Команды для работы с UI состоянием ======

#[tauri::command]
pub fn get_ui_state() -> Result<UIState, String> {
    let ui_state = API.config.get_ui_state().map_err(|e| e.to_string())?;

    tracing::debug!("Returning UI state: {} tabs, active_tab_id={:?}",
        ui_state.tabs.len(), ui_state.active_tab_id);

    Ok(ui_state)
}

#[tauri::command]
pub fn save_ui_state(ui_state: UIState) -> Result<(), String> {
    tracing::debug!("Saving UI state: {} tabs, active_tab_id={:?}",
        ui_state.tabs.len(), ui_state.active_tab_id);

    API.config.save_ui_state(ui_state).map_err(|e| e.to_string())
}

// ====== Команды для операций с прогрессом ======

/// Копирование файлов с прогрессом
#[tauri::command]
pub async fn copy_items_with_progress_command<R: Runtime>(
    app: AppHandle<R>,
    operation_id: String,
    sources: Vec<String>,
    destination: String,
    source_file_system: Option<String>,
    destination_file_system: Option<String>,
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
    match copy_items_with_progress(&sources, &destination, &tracker, &app, source_file_system, destination_file_system) {
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
    source_file_system: Option<String>,
    destination_file_system: Option<String>,
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
    match move_items_with_progress(&sources, &destination, &tracker, &app, source_file_system, destination_file_system) {
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
    panel_fs: Option<String>,
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
    match delete_items_with_progress(&paths, &tracker, &app, panel_fs) {
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

// ====== Команда для вычисления размера директории ======

#[derive(Serialize)]
pub struct DirectorySize {
    total_bytes: u64,
    total_items: u64,
}

#[tauri::command]
pub fn calculate_directory_size(path: String) -> Result<DirectorySize, String> {
    // For now, use the file_operations helper directly
    // TODO: Move to API service when implemented
    let (total_bytes, total_items) = calculate_total_size(&[path])
        .map_err(|e| format!("Failed to calculate directory size: {}", e))?;

    Ok(DirectorySize {
        total_bytes,
        total_items,
    })
}

// ====== Команды для мониторинга системы ======

use crate::api_service::models::SystemStats;

/// Получить статистику потребления ресурсов приложением
#[tauri::command]
pub async fn get_system_stats() -> Result<SystemStats, String> {
    API.system.get_stats().map_err(|e| e.to_string())
}

// ====== Conflict Resolution Commands ======

#[derive(Serialize)]
pub struct FileConflictInfo {
    source_path: String,
    destination_path: String,
    source_file: FileMetadata,
    destination_file: FileMetadata,
}

#[derive(Serialize)]
pub struct FileMetadata {
    name: String,
    size: u64,
    modified: u64,
}

#[tauri::command]
pub fn check_file_conflict(
    source_path: String,
    destination_dir: String,
    source_file_system: Option<String>,
    destination_file_system: Option<String>,
) -> Result<Option<FileConflictInfo>, String> {
    use std::path::Path;

    let source = Path::new(&source_path);

    // Получаем имя файла/папки
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid source path".to_string())?;

    // Формируем путь назначения
    let dest_path_buf = Path::new(&destination_dir).join(file_name);
    let dest_path = dest_path_buf.to_string_lossy().to_string();

    // Проверяем существует ли файл в месте назначения (using API for correct FS handling)
    let dest_info = match API.files.get_file_info(&dest_path, destination_file_system.as_deref()) {
        Ok(info) => info,
        Err(_) => return Ok(None), // Нет конфликта
    };

    // Получаем метаданные источника
    let source_info = API.files.get_file_info(&source_path, source_file_system.as_deref())
        .map_err(|e| format!("Failed to read source metadata: {}", e))?;

    // Создаем информацию о конфликте
    Ok(Some(FileConflictInfo {
        source_path: source_path.clone(),
        destination_path: dest_path,
        source_file: FileMetadata {
            name: file_name.to_string(),
            size: source_info.size.unwrap_or(0),
            modified: source_info.modified.unwrap_or(0),
        },
        destination_file: FileMetadata {
            name: file_name.to_string(),
            size: dest_info.size.unwrap_or(0),
            modified: dest_info.modified.unwrap_or(0),
        },
    }))
}

#[tauri::command]
pub fn copy_file_with_custom_name(
    source_path: String,
    destination_dir: String,
    new_name: String,
    source_file_system: Option<String>,
    destination_file_system: Option<String>,
) -> Result<(), String> {
    API.files
        .copy_with_custom_name(&source_path, &destination_dir, &new_name, source_file_system.as_deref(), destination_file_system.as_deref())
        .map_err(|e| e.to_string())
}

// ====== Batch Operations Commands ======

use crate::api_service::models::{PermissionsChange, DateChange, TagsChange};

/// Change file attributes (permissions, dates, tags)
#[tauri::command]
pub fn batch_change_attributes(
    path: String,
    permissions: Option<PermissionsChange>,
    dates: Option<DateChange>,
    tags: Option<TagsChange>,
) -> Result<(), String> {
    API.batch.change_attributes(&path, permissions, dates, tags)
        .map_err(|e| e.to_string())
}

/// Validate batch rename operation
#[tauri::command]
pub fn validate_batch_rename(new_names: Vec<String>) -> Result<Vec<String>, String> {
    API.batch.validate_rename(&new_names)
        .map_err(|e| e.to_string())
}

// ====== Vault Security Commands ======

#[tauri::command]
pub fn vault_is_enabled() -> Result<bool, String> {
    API.vault.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_get_status() -> Result<String, String> {
    API.vault.get_status().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_initialize(password: String) -> Result<(), String> {
    API.vault.initialize(password).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_unlock(password: String) -> Result<(), String> {
    API.vault.unlock(password).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_lock() -> Result<(), String> {
    API.vault.lock().map_err(|e| e.to_string())
}

// ====== Vault Recovery Commands ======

#[tauri::command]
pub fn vault_setup_recovery(channels: Vec<serde_json::Value>) -> Result<String, String> {
    use crate::api::notification_channels::ChannelConfig;

    let channel_configs: Vec<ChannelConfig> = channels
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();

    API.vault.setup_recovery(channel_configs).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_request_password_reset(channel_type: String) -> Result<(), String> {
    API.vault.request_password_reset(channel_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_verify_reset_code(code: String, new_password: String) -> Result<(), String> {
    API.vault.verify_reset_code(code, new_password).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_get_recovery_channels() -> Result<Vec<String>, String> {
    API.vault.get_recovery_channels().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_is_recovery_configured() -> Result<bool, String> {
    API.vault.is_recovery_configured().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_reset() -> Result<(), String> {
    API.vault.reset().map_err(|e| e.to_string())
}

// ====== Steganography Commands ======

#[tauri::command]
pub fn vault_create_stego_container(
    host_path: String,
    output_path: String,
    password: String,
) -> Result<(), String> {
    API.vault.create_stego_container(host_path, output_path, password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_create_container(
    source_path: String,
    output_path: String,
    password: String,
) -> Result<(), String> {
    API.vault.create_container(source_path, output_path, password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_create_new_secure_folder(
    name: String,
    parent_path: String,
    password: String,
) -> Result<(), String> {
    API.vault.create_new_secure_folder(name, parent_path, password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_hide_path_in_container(
    source_path: String,
    host_path: String,
    output_path: String,
    password: String,
) -> Result<(), String> {
    API.vault.hide_path_in_container(source_path, host_path, output_path, password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_extract_from_container(
    container_path: String,
    output_path: String,
    password: String,
) -> Result<(), String> {
    API.vault.extract_from_container(container_path, output_path, password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_open_stego_container(
    container_path: String,
    password: String,
) -> Result<String, String> {
    API.vault.open_stego_container(container_path, password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vault_save_stego_container(
    session_id: String,
) -> Result<(), String> {
    API.vault.save_stego_container(session_id)
        .map_err(|e| e.to_string())
}

// ====== Vault Directory Management Commands ======

#[tauri::command]
pub fn vault_get_current_directory() -> Result<String, String> {
    let config = crate::state::APP_CONFIG.read().unwrap();
    config.get_vault_dir()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn vault_get_default_directory() -> Result<String, String> {
    crate::config::AppConfig::default_vault_dir()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn vault_select_directory() -> Result<Option<String>, String> {
    let folder = rfd::AsyncFileDialog::new()
        .set_title("Select Vault Directory")
        .pick_folder()
        .await;

    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn vault_select_file() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .set_title("Select File")
        .pick_file()
        .await;

    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn vault_save_file_dialog() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .set_title("Save File")
        .save_file()
        .await;

    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub fn vault_set_custom_directory(path: String, migrate_data: bool) -> Result<(), String> {
    use std::path::Path;

    let new_path = Path::new(&path);

    // Validate path
    if !new_path.exists() {
        return Err("Selected directory does not exist".to_string());
    }

    if !new_path.is_dir() {
        return Err("Selected path is not a directory".to_string());
    }

    // Get current paths before changing config
    let old_paths = {
        let config = crate::state::APP_CONFIG.read().unwrap();
        config.get_vault_paths().map_err(|e| e.to_string())?
    };

    // Update config
    {
        let mut config = crate::state::APP_CONFIG.write().unwrap();
        config.vault.custom_path = Some(path.clone());
        config.vault.use_custom_path = true;
        config.save().map_err(|e| e.to_string())?;
    }

    // Get new paths after config change
    let new_paths = {
        let config = crate::state::APP_CONFIG.read().unwrap();
        config.get_vault_paths().map_err(|e| e.to_string())?
    };

    // Perform migration if requested
    if migrate_data {
        migrate_vault_files(&old_paths, &new_paths)?;
    }

    // Clear VFS from memory to force reinitialization
    {
        use crate::api_service::vault::VAULT_FS;
        let mut vfs_guard = VAULT_FS.lock().unwrap();
        *vfs_guard = None;
    }

    Ok(())
}

#[tauri::command]
pub fn vault_reset_to_default_directory(migrate_data: bool) -> Result<(), String> {
    // Get current paths
    let old_paths = {
        let config = crate::state::APP_CONFIG.read().unwrap();
        config.get_vault_paths().map_err(|e| e.to_string())?
    };

    // Update config to use default
    {
        let mut config = crate::state::APP_CONFIG.write().unwrap();
        config.vault.use_custom_path = false;
        config.vault.custom_path = None;
        config.save().map_err(|e| e.to_string())?;
    }

    // Get new default paths
    let new_paths = {
        let config = crate::state::APP_CONFIG.read().unwrap();
        config.get_vault_paths().map_err(|e| e.to_string())?
    };

    // Perform migration if requested
    if migrate_data {
        migrate_vault_files(&old_paths, &new_paths)?;
    }

    // Clear VFS from memory
    {
        use crate::api_service::vault::VAULT_FS;
        let mut vfs_guard = VAULT_FS.lock().unwrap();
        *vfs_guard = None;
    }

    Ok(())
}

/// Helper function to migrate vault files
fn migrate_vault_files(old_paths: &crate::config::VaultPaths, new_paths: &crate::config::VaultPaths) -> Result<(), String> {
    use std::fs;

    // Skip migration if source and destination are the same
    if old_paths.dir == new_paths.dir {
        tracing::info!("Source and destination are the same, skipping migration");
        return Ok(());
    }

    // Check if source files exist
    if !old_paths.fs_json.exists() && !old_paths.vault_meta.exists() && !old_paths.vault_bin.exists() {
        tracing::info!("No vault files to migrate");
        return Ok(());
    }

    let files_to_migrate = vec![
        (&old_paths.fs_json, &new_paths.fs_json, "fs.json"),
        (&old_paths.vault_meta, &new_paths.vault_meta, "vault.meta"),
        (&old_paths.vault_bin, &new_paths.vault_bin, "vault.bin"),
    ];

    // Create destination directory
    if !new_paths.dir.exists() {
        fs::create_dir_all(&new_paths.dir)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    }

    // Check for conflicts
    for (_, dst, name) in &files_to_migrate {
        if dst.exists() {
            return Err(format!(
                "Destination file already exists: {}. Please manually resolve the conflict.",
                name
            ));
        }
    }

    // Copy files with verification
    for (src, dst, name) in &files_to_migrate {
        if src.exists() {
            // Copy file
            fs::copy(src, dst)
                .map_err(|e| format!("Failed to copy {}: {}", name, e))?;

            // Verify file size matches
            let src_size = src.metadata()
                .map_err(|e| format!("Failed to read source metadata for {}: {}", name, e))?
                .len();
            let dst_size = dst.metadata()
                .map_err(|e| format!("Failed to read destination metadata for {}: {}", name, e))?
                .len();

            if src_size != dst_size {
                // Rollback on verification failure
                let _ = fs::remove_file(dst);
                return Err(format!("File size mismatch after copying {}", name));
            }

            tracing::info!("Migrated {} -> {}", src.display(), dst.display());
        }
    }

    // Delete old files only after ALL files are successfully copied and verified
    for (src, _, name) in &files_to_migrate {
        if src.exists() {
            fs::remove_file(src)
                .map_err(|e| format!("Failed to remove old file {}: {}", name, e))?;
        }
    }

    tracing::info!("Vault migration completed from {:?} to {:?}", old_paths.dir, new_paths.dir);
    Ok(())
}

// ====== Queue Management Commands ======

use crate::queue::{
    QUEUE_MANAGER, QueuedOperation, OperationParams, OperationPriority, QueueConfig,
    QueuedOperationType,
};

#[tauri::command]
pub fn queue_add_operation(
    operation_type: String,
    params: serde_json::Value,
    priority: Option<String>,
    scheduled_at: Option<String>,
    retry_enabled: Option<bool>,
    description: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String> {
    use tracing::info;

    info!("queue_add_operation called with type: {}", operation_type);
    info!("params: {}", serde_json::to_string_pretty(&params).unwrap_or_default());

    // Parse operation type
    let op_type: QueuedOperationType = serde_json::from_value(serde_json::json!(operation_type))
        .map_err(|e| {
            let err_msg = format!("Invalid operation type '{}': {}", operation_type, e);
            tracing::error!("{}", err_msg);
            err_msg
        })?;

    // Parse parameters
    let op_params: OperationParams = serde_json::from_value(params.clone())
        .map_err(|e| {
            let err_msg = format!("Invalid operation params: {}. Params were: {}", e, serde_json::to_string(&params).unwrap_or_default());
            tracing::error!("{}", err_msg);
            err_msg
        })?;

    // Parse priority
    let priority = match priority.as_deref() {
        Some("low") => OperationPriority::Low,
        Some("high") => OperationPriority::High,
        Some("urgent") => OperationPriority::Urgent,
        _ => OperationPriority::Normal,
    };

    // Create operation
    let mut operation = QueuedOperation::new(op_type, op_params, priority);

    // Set scheduled time if provided
    if let Some(scheduled_str) = scheduled_at {
        let scheduled_dt = chrono::DateTime::parse_from_rfc3339(&scheduled_str)
            .map_err(|e| format!("Invalid scheduled time: {}", e))?;
        operation.scheduled_at = Some(scheduled_dt.with_timezone(&chrono::Utc));
    }

    // Set retry policy
    if let Some(enabled) = retry_enabled {
        operation.retry_policy.enabled = enabled;
    }

    // Set metadata
    if let Some(desc) = description {
        operation.description = Some(desc);
    }
    if let Some(t) = tags {
        operation.tags = t;
    }

    QUEUE_MANAGER.enqueue(operation)
}

#[tauri::command]
pub fn queue_get_all_operations() -> Result<Vec<QueuedOperation>, String> {
    Ok(QUEUE_MANAGER.get_all_operations())
}

#[tauri::command]
pub fn queue_get_operation(operation_id: String) -> Result<Option<QueuedOperation>, String> {
    Ok(QUEUE_MANAGER.get_operation(&operation_id))
}

#[tauri::command]
pub fn queue_cancel_operation(operation_id: String) -> Result<(), String> {
    QUEUE_MANAGER.cancel_operation(&operation_id)
}

#[tauri::command]
pub fn queue_retry_operation(operation_id: String) -> Result<(), String> {
    QUEUE_MANAGER.retry_operation(&operation_id)
}

#[tauri::command]
pub fn queue_remove_operation(operation_id: String) -> Result<(), String> {
    QUEUE_MANAGER.remove_operation(&operation_id)
}

#[tauri::command]
pub fn queue_update_config(config: QueueConfig) -> Result<(), String> {
    QUEUE_MANAGER.update_config(config);
    Ok(())
}

#[tauri::command]
pub fn queue_get_config() -> Result<QueueConfig, String> {
    Ok(QUEUE_MANAGER.get_config())
}

#[tauri::command]
pub fn queue_pause_operation(operation_id: String) -> Result<(), String> {
    QUEUE_MANAGER.pause_operation(&operation_id)
}

#[tauri::command]
pub fn queue_resume_operation(operation_id: String) -> Result<(), String> {
    QUEUE_MANAGER.resume_operation(&operation_id)
}

#[tauri::command]
pub fn queue_run_now(operation_id: String) -> Result<(), String> {
    QUEUE_MANAGER.run_now(&operation_id)
}
