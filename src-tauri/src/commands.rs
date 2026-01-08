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

#[derive(Serialize)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

#[tauri::command]
pub fn execute_command(
    command: String,
    working_dir: String,
) -> Result<CommandResult, String> {
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use wait_timeout::ChildExt;

    // Spawn the process
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .current_dir(&working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start command: {}", e))?;

    // Wait with 30 second timeout
    let timeout = Duration::from_secs(30);
    match child.wait_timeout(timeout)
        .map_err(|e| format!("Failed to wait for command: {}", e))? {
        Some(_status) => {
            // Process finished within timeout, get output
            let output = child.wait_with_output()
                .map_err(|e| format!("Failed to get command output: {}", e))?;

            Ok(CommandResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                success: output.status.success(),
            })
        }
        None => {
            // Timeout expired, kill the process
            let _ = child.kill();
            let _ = child.wait(); // Reap the zombie process

            Ok(CommandResult {
                stdout: String::new(),
                stderr: "Command timed out after 30 seconds. Long-running commands are not supported.".to_string(),
                exit_code: -1,
                success: false,
            })
        }
    }
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
pub fn list_archive_contents(archive_path: String) -> Result<Vec<FileSystemEntry>, String> {
    crate::archives::list_archive_contents(&archive_path)
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

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PermissionsChange {
    pub readable: Option<bool>,
    pub writable: Option<bool>,
    pub executable: Option<bool>,
    pub recursive: bool,
}

#[derive(Debug, Deserialize)]
pub struct DateChange {
    pub modified: Option<u64>,
    pub created: Option<u64>,
    pub accessed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TagsChange {
    pub operation: String, // "add", "remove", "replace"
    pub tags: Vec<String>,
}

/// Change file attributes (permissions, dates, tags)
#[tauri::command]
pub fn batch_change_attributes(
    path: String,
    permissions: Option<PermissionsChange>,
    dates: Option<DateChange>,
    tags: Option<TagsChange>,
) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let file_path = Path::new(&path);

    // Change permissions
    #[cfg(unix)]
    if let Some(perms) = permissions {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        let mut mode = metadata.permissions().mode();

        // Update permission bits
        if let Some(readable) = perms.readable {
            if readable {
                mode |= 0o444; // r--r--r--
            } else {
                mode &= !0o444;
            }
        }

        if let Some(writable) = perms.writable {
            if writable {
                mode |= 0o222; // -w--w--w-
            } else {
                mode &= !0o222;
            }
        }

        if let Some(executable) = perms.executable {
            if executable {
                mode |= 0o111; // --x--x--x
            } else {
                mode &= !0o111;
            }
        }

        let new_permissions = fs::Permissions::from_mode(mode);
        fs::set_permissions(file_path, new_permissions)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    // Change dates
    if let Some(date_changes) = dates {
        use filetime::{set_file_times, FileTime};

        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        let current_accessed = FileTime::from_last_access_time(&metadata);
        let current_modified = FileTime::from_last_modification_time(&metadata);

        let new_accessed = if let Some(accessed) = date_changes.accessed {
            FileTime::from_unix_time(accessed as i64, 0)
        } else {
            current_accessed
        };

        let new_modified = if let Some(modified) = date_changes.modified {
            FileTime::from_unix_time(modified as i64, 0)
        } else {
            current_modified
        };

        set_file_times(file_path, new_accessed, new_modified)
            .map_err(|e| format!("Failed to set file times: {}", e))?;
    }

    // Change tags (macOS extended attributes or custom metadata)
    #[cfg(target_os = "macos")]
    if let Some(tags_change) = tags {
        use std::process::Command;

        let tags_str = tags_change.tags.join(",");

        match tags_change.operation.as_str() {
            "add" | "replace" => {
                // Use xattr command to set tags
                Command::new("xattr")
                    .args([
                        "-w",
                        "com.apple.metadata:_kMDItemUserTags",
                        &tags_str,
                        &path,
                    ])
                    .output()
                    .map_err(|e| format!("Failed to set tags: {}", e))?;
            }
            "remove" => {
                // Remove tags attribute
                Command::new("xattr")
                    .args(["-d", "com.apple.metadata:_kMDItemUserTags", &path])
                    .output()
                    .ok(); // Ignore errors if attribute doesn't exist
            }
            _ => return Err("Invalid tag operation".to_string()),
        }
    }

    Ok(())
}

/// Validate batch rename operation
#[tauri::command]
pub fn validate_batch_rename(new_names: Vec<String>) -> Result<Vec<String>, String> {
    use std::collections::HashSet;

    let mut errors = Vec::new();
    let mut seen_names = HashSet::new();

    for name in &new_names {
        // Check for empty names
        if name.trim().is_empty() {
            errors.push(format!("Empty filename is not allowed"));
            continue;
        }

        // Check for illegal characters
        let illegal_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        if name.chars().any(|c| illegal_chars.contains(&c)) {
            errors.push(format!("Filename '{}' contains illegal characters", name));
        }

        // Check for duplicate names
        let lower_name = name.to_lowercase();
        if seen_names.contains(&lower_name) {
            errors.push(format!("Duplicate filename: '{}'", name));
        }
        seen_names.insert(lower_name);
    }

    if errors.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(errors)
    }
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
