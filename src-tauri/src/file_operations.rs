use crate::core::FileSystemError;
use crate::progress::{emit_progress, ProgressTracker};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::AppHandle;

const BUFFER_SIZE: usize = 64 * 1024; // 64KB buffer

/// Проверяет паузу и отмену операции
fn check_pause_and_cancel<R: tauri::Runtime>(
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    // Ждем пока на паузе
    while tracker.is_paused() {
        if tracker.is_cancelled() {
            return Err(FileSystemError::new("Operation cancelled"));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        emit_progress(app, tracker);
    }

    // Проверяем отмену
    if tracker.is_cancelled() {
        return Err(FileSystemError::new("Operation cancelled"));
    }

    Ok(())
}

/// Вычисляет общий размер файлов/директорий
pub fn calculate_total_size(paths: &[String]) -> io::Result<(u64, u64)> {
    let mut total_bytes = 0u64;
    let mut total_items = 0u64;

    for path_str in paths {
        let path = Path::new(path_str);
        if path.is_file() {
            total_bytes += fs::metadata(path)?.len();
            total_items += 1;
        } else if path.is_dir() {
            let (bytes, items) = calculate_dir_size(path)?;
            total_bytes += bytes;
            total_items += items;
        }
    }

    Ok((total_bytes, total_items))
}

/// Рекурсивно вычисляет размер директории
fn calculate_dir_size(path: &Path) -> io::Result<(u64, u64)> {
    let mut total_bytes = 0u64;
    let mut total_items = 0u64;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            total_bytes += fs::metadata(&path)?.len();
            total_items += 1;
        } else if path.is_dir() {
            let (bytes, items) = calculate_dir_size(&path)?;
            total_bytes += bytes;
            total_items += items;
        }
    }

    Ok((total_bytes, total_items))
}

/// Копирует файл с прогрессом
pub fn copy_file_with_progress<R: tauri::Runtime>(
    src: &Path,
    dest: &Path,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    // Проверяем отмену
    if tracker.is_cancelled() {
        return Err(FileSystemError::new("Operation cancelled"));
    }

    // Обновляем текущий файл
    tracker.update_current_file(Some(
        src.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
    ));

    let mut source = fs::File::open(src)
        .map_err(|e| FileSystemError::new(format!("Failed to open source: {}", e)))?;

    let mut destination = fs::File::create(dest)
        .map_err(|e| FileSystemError::new(format!("Failed to create destination: {}", e)))?;

    let mut buffer = vec![0; BUFFER_SIZE];

    loop {
        // Проверяем паузу и отмену
        check_pause_and_cancel(tracker, app)?;

        let bytes_read = source
            .read(&mut buffer)
            .map_err(|e| FileSystemError::new(format!("Failed to read: {}", e)))?;

        if bytes_read == 0 {
            break;
        }

        destination
            .write_all(&buffer[..bytes_read])
            .map_err(|e| FileSystemError::new(format!("Failed to write: {}", e)))?;

        tracker.add_bytes(bytes_read as u64);

        // Отправляем обновление прогресса
        emit_progress(app, tracker);
    }

    // Копируем метаданные (permissions, timestamps)
    if let Ok(metadata) = fs::metadata(src) {
        let _ = fs::set_permissions(dest, metadata.permissions());
    }

    tracker.add_item();

    Ok(())
}

/// Рекурсивно копирует директорию с прогрессом
pub fn copy_dir_with_progress<R: tauri::Runtime>(
    src: &Path,
    dest: &Path,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    // Проверяем паузу и отмену
    check_pause_and_cancel(tracker, app)?;

    fs::create_dir_all(dest)
        .map_err(|e| FileSystemError::new(format!("Failed to create directory: {}", e)))?;

    for entry in fs::read_dir(src)
        .map_err(|e| FileSystemError::new(format!("Failed to read directory: {}", e)))?
    {
        let entry = entry
            .map_err(|e| FileSystemError::new(format!("Failed to read entry: {}", e)))?;

        let path = entry.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| FileSystemError::new("Could not get file name"))?;
        let dest_path = dest.join(file_name);

        if path.is_dir() {
            copy_dir_with_progress(&path, &dest_path, tracker, app)?;
        } else {
            copy_file_with_progress(&path, &dest_path, tracker, app)?;
        }
    }

    Ok(())
}

/// Копирует элементы с прогрессом
pub fn copy_items_with_progress<R: tauri::Runtime>(
    sources: &[String],
    destination: &str,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    let dest_path = PathBuf::from(destination);

    if !dest_path.exists() || !dest_path.is_dir() {
        return Err(FileSystemError::new(format!(
            "Destination is not a valid directory: {}",
            destination
        )));
    }

    for source in sources {
        // Проверяем паузу и отмену
        check_pause_and_cancel(tracker, app)?;

        let source_path = PathBuf::from(source);

        if !source_path.exists() {
            return Err(FileSystemError::new(format!(
                "Source does not exist: {}",
                source
            )));
        }

        let file_name = source_path
            .file_name()
            .ok_or_else(|| FileSystemError::new("Could not get file name"))?;

        let dest_file_path = dest_path.join(file_name);

        if source_path.is_dir() {
            copy_dir_with_progress(&source_path, &dest_file_path, tracker, app)?;
        } else {
            copy_file_with_progress(&source_path, &dest_file_path, tracker, app)?;
        }
    }

    Ok(())
}

/// Перемещает элементы с прогрессом
pub fn move_items_with_progress<R: tauri::Runtime>(
    sources: &[String],
    destination: &str,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    let dest_path = PathBuf::from(destination);

    if !dest_path.exists() || !dest_path.is_dir() {
        return Err(FileSystemError::new(format!(
            "Destination is not a valid directory: {}",
            destination
        )));
    }

    for source in sources {
        // Проверяем паузу и отмену
        check_pause_and_cancel(tracker, app)?;

        let source_path = PathBuf::from(source);

        if !source_path.exists() {
            return Err(FileSystemError::new(format!(
                "Source does not exist: {}",
                source
            )));
        }

        let file_name = source_path
            .file_name()
            .ok_or_else(|| FileSystemError::new("Could not get file name"))?;

        let dest_file_path = dest_path.join(file_name);

        tracker.update_current_file(Some(file_name.to_str().unwrap_or("").to_string()));

        // Пробуем простое переименование (быстрее, если на том же разделе)
        if fs::rename(&source_path, &dest_file_path).is_ok() {
            // Для переименования добавляем размер файла к прогрессу
            if let Ok(metadata) = fs::metadata(&dest_file_path) {
                if metadata.is_file() {
                    tracker.add_bytes(metadata.len());
                }
            }
            tracker.add_item();
            emit_progress(app, tracker);
        } else {
            // Если не получилось переименовать, копируем и удаляем
            if source_path.is_dir() {
                copy_dir_with_progress(&source_path, &dest_file_path, tracker, app)?;
                fs::remove_dir_all(&source_path).map_err(|e| {
                    FileSystemError::new(format!("Failed to remove source directory: {}", e))
                })?;
            } else {
                copy_file_with_progress(&source_path, &dest_file_path, tracker, app)?;
                fs::remove_file(&source_path).map_err(|e| {
                    FileSystemError::new(format!("Failed to remove source file: {}", e))
                })?;
            }
        }
    }

    Ok(())
}

/// Удаляет файл с прогрессом
fn delete_file_with_progress<R: tauri::Runtime>(
    path: &Path,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    // Проверяем паузу и отмену
    check_pause_and_cancel(tracker, app)?;

    tracker.update_current_file(Some(
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
    ));

    let metadata = fs::metadata(path)
        .map_err(|e| FileSystemError::new(format!("Failed to read metadata: {}", e)))?;

    fs::remove_file(path)
        .map_err(|e| FileSystemError::new(format!("Failed to delete file: {}", e)))?;

    tracker.add_bytes(metadata.len());
    tracker.add_item();
    emit_progress(app, tracker);

    Ok(())
}

/// Рекурсивно удаляет директорию с прогрессом
fn delete_dir_with_progress<R: tauri::Runtime>(
    path: &Path,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    // Проверяем паузу и отмену
    check_pause_and_cancel(tracker, app)?;

    for entry in fs::read_dir(path)
        .map_err(|e| FileSystemError::new(format!("Failed to read directory: {}", e)))?
    {
        let entry = entry
            .map_err(|e| FileSystemError::new(format!("Failed to read entry: {}", e)))?;

        let entry_path = entry.path();

        if entry_path.is_dir() {
            delete_dir_with_progress(&entry_path, tracker, app)?;
        } else {
            delete_file_with_progress(&entry_path, tracker, app)?;
        }
    }

    fs::remove_dir(path)
        .map_err(|e| FileSystemError::new(format!("Failed to delete directory: {}", e)))?;

    Ok(())
}

/// Удаляет элементы с прогрессом
pub fn delete_items_with_progress<R: tauri::Runtime>(
    paths: &[String],
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    for path_str in paths {
        // Проверяем паузу и отмену
        check_pause_and_cancel(tracker, app)?;

        let path = PathBuf::from(path_str);

        if !path.exists() {
            return Err(FileSystemError::new(format!(
                "Path does not exist: {}",
                path_str
            )));
        }

        if path.is_dir() {
            delete_dir_with_progress(&path, tracker, app)?;
        } else {
            delete_file_with_progress(&path, tracker, app)?;
        }
    }

    Ok(())
}
