use crate::core::FileSystemError;
use crate::progress::{emit_progress, ProgressTracker};
use crate::api_service::API;
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

/// Вычисляет общий размер файлов/директорий (только для Real FS пока)
pub fn calculate_total_size(paths: &[String]) -> io::Result<(u64, u64)> {
    let mut total_bytes = 0u64;
    let mut total_items = 0u64;

    for path_str in paths {
        let path = Path::new(path_str);
        if path.exists() {
            if path.is_file() {
                total_bytes += fs::metadata(path)?.len();
                total_items += 1;
            } else if path.is_dir() {
                let (bytes, items) = calculate_dir_size(path)?;
                total_bytes += bytes;
                total_items += items;
            }
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

/// Копирует файл с прогрессом (Real FS)
pub fn copy_file_real_fs<R: tauri::Runtime>(
    src: &Path,
    dest: &Path,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    if tracker.is_cancelled() {
        return Err(FileSystemError::new("Operation cancelled"));
    }

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
        emit_progress(app, tracker);
    }

    if let Ok(metadata) = fs::metadata(src) {
        let _ = fs::set_permissions(dest, metadata.permissions());
    }

    tracker.add_item();
    Ok(())
}

/// Рекурсивно копирует директорию с прогрессом (Real FS)
pub fn copy_dir_real_fs<R: tauri::Runtime>(
    src: &Path,
    dest: &Path,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
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
            copy_dir_real_fs(&path, &dest_path, tracker, app)?;
        } else {
            copy_file_real_fs(&path, &dest_path, tracker, app)?;
        }
    }

    Ok(())
}

/// Helper for Cross-FS Copy
fn copy_recursive_cross_fs<R: tauri::Runtime>(
    source_path: &str,
    dest_parent: &str,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
    source_fs: Option<&str>,
    dest_fs: Option<&str>,
) -> Result<(), FileSystemError> {
    check_pause_and_cancel(tracker, app)?;

    // Get info about source
    let info = API.files.get_file_info(source_path, source_fs)
        .map_err(|e| FileSystemError::new(e.to_string()))?;

    let name = &info.name;
    let dest_path_buf = Path::new(dest_parent).join(name);
    let dest_path = dest_path_buf.to_string_lossy().to_string();

    tracker.update_current_file(Some(name.clone()));

    if info.is_dir {
        // Create directory in destination
        API.files.create_folder(dest_parent, name, dest_fs)
            .map_err(|e| FileSystemError::new(format!("Failed to create folder: {}", e)))?;

        // List contents
        let entries = API.files.list_directory(source_path, source_fs)
            .map_err(|e| FileSystemError::new(format!("Failed to list directory: {}", e)))?;

        for entry in entries {
            copy_recursive_cross_fs(&entry.path, &dest_path, tracker, app, source_fs, dest_fs)?;
        }
    } else {
        // Read content (binary safe)
        let content = API.files.read_file_bytes(source_path, source_fs)
            .map_err(|e| FileSystemError::new(format!("Failed to read file content: {}", e)))?;

        // Write content (binary safe) using write_file_bytes which expects full path
        // We already constructed dest_path above as full path to destination file
        API.files.write_file_bytes(&dest_path, &content, dest_fs)
            .map_err(|e| FileSystemError::new(format!("Failed to write file: {}", e)))?;

        // Update progress
        tracker.add_bytes(content.len() as u64);
        tracker.add_item();
        emit_progress(app, tracker);
    }

    Ok(())
}

/// Копирует элементы с прогрессом
pub fn copy_items_with_progress<R: tauri::Runtime>(
    sources: &[String],
    destination: &str,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
    source_fs: Option<String>,
    dest_fs: Option<String>,
) -> Result<(), FileSystemError> {
    let is_real_source = source_fs.is_none() || source_fs.as_deref() == Some("real");
    let is_real_dest = dest_fs.is_none() || dest_fs.as_deref() == Some("real");

    if is_real_source && is_real_dest {
        // Use optimized Real FS implementation
        let dest_path = PathBuf::from(destination);
        if !dest_path.exists() || !dest_path.is_dir() {
            return Err(FileSystemError::new(format!("Destination is not a valid directory: {}", destination)));
        }

        for source in sources {
            check_pause_and_cancel(tracker, app)?;
            let source_path = PathBuf::from(source);
            if !source_path.exists() {
                return Err(FileSystemError::new(format!("Source does not exist: {}", source)));
            }
            let file_name = source_path.file_name().ok_or_else(|| FileSystemError::new("Could not get file name"))?;
            let dest_file_path = dest_path.join(file_name);

            if source_path.is_dir() {
                copy_dir_real_fs(&source_path, &dest_file_path, tracker, app)?;
            } else {
                copy_file_real_fs(&source_path, &dest_file_path, tracker, app)?;
            }
        }
    } else {
        // Cross-FS implementation
        for source in sources {
            copy_recursive_cross_fs(source, destination, tracker, app, source_fs.as_deref(), dest_fs.as_deref())?;
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
    source_fs: Option<String>,
    dest_fs: Option<String>,
) -> Result<(), FileSystemError> {
    let is_real_source = source_fs.is_none() || source_fs.as_deref() == Some("real");
    let is_real_dest = dest_fs.is_none() || dest_fs.as_deref() == Some("real");

    if is_real_source && is_real_dest {
        // Real FS Move optimization
        let dest_path = PathBuf::from(destination);
        if !dest_path.exists() || !dest_path.is_dir() {
            return Err(FileSystemError::new(format!("Destination is not a valid directory: {}", destination)));
        }

        for source in sources {
            check_pause_and_cancel(tracker, app)?;
            let source_path = PathBuf::from(source);
            if !source_path.exists() {
                return Err(FileSystemError::new(format!("Source does not exist: {}", source)));
            }
            let file_name = source_path.file_name().ok_or_else(|| FileSystemError::new("Could not get file name"))?;
            let dest_file_path = dest_path.join(file_name);

            tracker.update_current_file(Some(file_name.to_str().unwrap_or("").to_string()));

            if fs::rename(&source_path, &dest_file_path).is_ok() {
                if let Ok(metadata) = fs::metadata(&dest_file_path) {
                    if metadata.is_file() {
                        tracker.add_bytes(metadata.len());
                    }
                }
                tracker.add_item();
                emit_progress(app, tracker);
            } else {
                if source_path.is_dir() {
                    copy_dir_real_fs(&source_path, &dest_file_path, tracker, app)?;
                    fs::remove_dir_all(&source_path).map_err(|e| FileSystemError::new(format!("Failed to remove source directory: {}", e)))?;
                } else {
                    copy_file_real_fs(&source_path, &dest_file_path, tracker, app)?;
                    fs::remove_file(&source_path).map_err(|e| FileSystemError::new(format!("Failed to remove source file: {}", e)))?;
                }
            }
        }
    } else {
        // Cross-FS Move: Copy then Delete
        for source in sources {
            check_pause_and_cancel(tracker, app)?;
            
            // 1. Copy
            copy_recursive_cross_fs(source, destination, tracker, app, source_fs.as_deref(), dest_fs.as_deref())?;
            
            // 2. Delete source
            API.files.delete_item(source, source_fs.as_deref())
                .map_err(|e| FileSystemError::new(format!("Failed to delete source after move: {}", e)))?;
        }
    }

    Ok(())
}

/// Удаляет элементы с прогрессом
pub fn delete_items_with_progress<R: tauri::Runtime>(
    paths: &[String],
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
    panel_fs: Option<String>,
) -> Result<(), FileSystemError> {
    let is_real_fs = panel_fs.is_none() || panel_fs.as_deref() == Some("real");

    if is_real_fs {
        for path_str in paths {
            check_pause_and_cancel(tracker, app)?;
            let path = PathBuf::from(path_str);
            if !path.exists() {
                return Err(FileSystemError::new(format!("Path does not exist: {}", path_str)));
            }
            if path.is_dir() {
                delete_dir_real_fs(&path, tracker, app)?;
            } else {
                delete_file_real_fs(&path, tracker, app)?;
            }
        }
    } else {
        // Virtual FS Delete
        for path in paths {
            check_pause_and_cancel(tracker, app)?;
            tracker.update_current_file(Some(path.clone()));
            
            // Note: API.files.delete_item is usually recursive.
            // We might not get granular progress here unless we recurse manually.
            // For now, we trust delete_item.
            API.files.delete_item(path, panel_fs.as_deref())
                .map_err(|e| FileSystemError::new(format!("Failed to delete item: {}", e)))?;
            
            tracker.add_item();
            emit_progress(app, tracker);
        }
    }

    Ok(())
}

fn delete_file_real_fs<R: tauri::Runtime>(
    path: &Path,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    check_pause_and_cancel(tracker, app)?;
    tracker.update_current_file(Some(path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string()));
    
    let len = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    fs::remove_file(path).map_err(|e| FileSystemError::new(format!("Failed to delete file: {}", e)))?;
    
    tracker.add_bytes(len);
    tracker.add_item();
    emit_progress(app, tracker);
    Ok(())
}

fn delete_dir_real_fs<R: tauri::Runtime>(
    path: &Path,
    tracker: &Arc<ProgressTracker>,
    app: &AppHandle<R>,
) -> Result<(), FileSystemError> {
    check_pause_and_cancel(tracker, app)?;
    
    for entry in fs::read_dir(path).map_err(|e| FileSystemError::new(format!("Failed to read dir: {}", e)))? {
        let entry = entry.map_err(|e| FileSystemError::new(format!("Failed to read entry: {}", e)))?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            delete_dir_real_fs(&entry_path, tracker, app)?;
        } else {
            delete_file_real_fs(&entry_path, tracker, app)?;
        }
    }
    
    fs::remove_dir(path).map_err(|e| FileSystemError::new(format!("Failed to delete dir: {}", e)))?;
    Ok(())
}

// ====== Simplified versions for queue operations (without AppHandle) ======

/// Simplified pause/cancel check without emit
fn check_pause_and_cancel_simple(tracker: &Arc<ProgressTracker>) -> Result<(), FileSystemError> {
    // Wait while paused
    while tracker.is_paused() {
        if tracker.is_cancelled() {
            return Err(FileSystemError::new("Operation cancelled"));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Check cancellation
    if tracker.is_cancelled() {
        return Err(FileSystemError::new("Operation cancelled"));
    }

    Ok(())
}

/// Copy file (Real FS) without emit
fn copy_file_real_fs_simple(
    src: &Path,
    dest: &Path,
    tracker: &Arc<ProgressTracker>,
) -> Result<(), FileSystemError> {
    check_pause_and_cancel_simple(tracker)?;

    let mut source = fs::File::open(src)
        .map_err(|e| FileSystemError::new(format!("Failed to open source file: {}", e)))?;
    let mut destination = fs::File::create(dest)
        .map_err(|e| FileSystemError::new(format!("Failed to create destination file: {}", e)))?;

    let mut buffer = vec![0u8; BUFFER_SIZE];
    loop {
        check_pause_and_cancel_simple(tracker)?;

        let bytes_read = source
            .read(&mut buffer)
            .map_err(|e| FileSystemError::new(format!("Failed to read from source: {}", e)))?;
        if bytes_read == 0 {
            break;
        }

        destination
            .write_all(&buffer[..bytes_read])
            .map_err(|e| FileSystemError::new(format!("Failed to write to destination: {}", e)))?;

        tracker.add_bytes(bytes_read as u64);
    }

    tracker.add_item();
    Ok(())
}

/// Copy directory (Real FS) without emit
fn copy_dir_real_fs_simple(
    src: &Path,
    dest: &Path,
    tracker: &Arc<ProgressTracker>,
) -> Result<(), FileSystemError> {
    check_pause_and_cancel_simple(tracker)?;

    fs::create_dir_all(dest)
        .map_err(|e| FileSystemError::new(format!("Failed to create directory: {}", e)))?;

    for entry in fs::read_dir(src)
        .map_err(|e| FileSystemError::new(format!("Failed to read directory: {}", e)))?
    {
        let entry = entry.map_err(|e| FileSystemError::new(format!("Failed to read entry: {}", e)))?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);

        if entry_path.is_dir() {
            copy_dir_real_fs_simple(&entry_path, &dest_path, tracker)?;
        } else {
            copy_file_real_fs_simple(&entry_path, &dest_path, tracker)?;
        }

        tracker.add_item();
    }

    Ok(())
}

/// Generic recursive copy helper (Virtual/Cross-FS)
fn copy_recursive_generic(
    source_path: &str,
    dest_parent: &str,
    tracker: &Arc<ProgressTracker>,
    source_fs: Option<&str>,
    dest_fs: Option<&str>,
) -> Result<(), FileSystemError> {
    check_pause_and_cancel_simple(tracker)?;

    // Get info about source
    let info = API.files.get_file_info(source_path, source_fs)
        .map_err(|e| FileSystemError::new(e.to_string()))?;

    let name = &info.name;
    // Simple path join - assumes forward slash for now as VirtualFS standard
    // For RealFS mixed with VirtualFS, we might need better path handling, but this covers most cases
    let dest_path = if dest_parent.ends_with('/') || dest_parent.ends_with('\\') {
        format!("{}{}", dest_parent, name)
    } else {
        format!("{}/{}", dest_parent, name)
    };

    tracker.update_current_file(Some(name.clone()));

    if info.is_dir {
        // Create directory in destination
        API.files.create_folder(dest_parent, name, dest_fs)
            .map_err(|e| FileSystemError::new(format!("Failed to create folder: {}", e)))?;

        // List contents
        let entries = API.files.list_directory(source_path, source_fs)
            .map_err(|e| FileSystemError::new(format!("Failed to list directory: {}", e)))?;

        for entry in entries {
            copy_recursive_generic(&entry.path, &dest_path, tracker, source_fs, dest_fs)?;
        }
        
        // Add directory itself to item count
        tracker.add_item();
    } else {
        // Read content
        let content = API.files.read_file_bytes(source_path, source_fs)
            .map_err(|e| FileSystemError::new(format!("Failed to read file: {}", e)))?;

        // Write content
        API.files.write_file_bytes(&dest_path, &content, dest_fs)
            .map_err(|e| FileSystemError::new(format!("Failed to write file: {}", e)))?;

        // Update progress
        tracker.add_bytes(content.len() as u64);
        tracker.add_item();
    }

    Ok(())
}

/// Copy items without AppHandle (for queue operations)
pub fn copy_items_simple(
    sources: &[String],
    destination: &str,
    tracker: &Arc<ProgressTracker>,
    source_fs: Option<&str>,
    dest_fs: Option<&str>,
) -> Result<(), FileSystemError> {
    let is_real_source = source_fs.is_none() || source_fs == Some("real");
    let is_real_dest = dest_fs.is_none() || dest_fs == Some("real");

    if is_real_source && is_real_dest {
        // Use optimized Real FS implementation
        let dest_path = PathBuf::from(destination);
        if !dest_path.exists() || !dest_path.is_dir() {
            return Err(FileSystemError::new(format!(
                "Destination is not a valid directory: {}",
                destination
            )));
        }

        for source in sources {
            check_pause_and_cancel_simple(tracker)?;
            let source_path = PathBuf::from(source);
            if !source_path.exists() {
                return Err(FileSystemError::new(format!("Source does not exist: {}", source)));
            }
            let file_name = source_path
                .file_name()
                .ok_or_else(|| FileSystemError::new("Could not get file name"))?;
            let dest_file_path = dest_path.join(file_name);

            if source_path.is_dir() {
                copy_dir_real_fs_simple(&source_path, &dest_file_path, tracker)?;
            } else {
                copy_file_real_fs_simple(&source_path, &dest_file_path, tracker)?;
            }
        }
    } else {
        // Generic implementation using API service (supports Virtual & Cross-FS)
        for source in sources {
            copy_recursive_generic(source, destination, tracker, source_fs, dest_fs)?;
        }
    }

    Ok(())
}

/// Move items without AppHandle (for queue operations)
pub fn move_items_simple(
    sources: &[String],
    destination: &str,
    tracker: &Arc<ProgressTracker>,
    source_fs: Option<&str>,
    dest_fs: Option<&str>,
) -> Result<(), FileSystemError> {
    let is_real_source = source_fs.is_none() || source_fs == Some("real");
    let is_real_dest = dest_fs.is_none() || dest_fs == Some("real");

    if is_real_source && is_real_dest {
        let dest_path = PathBuf::from(destination);
        if !dest_path.exists() || !dest_path.is_dir() {
            return Err(FileSystemError::new(format!(
                "Destination is not a valid directory: {}",
                destination
            )));
        }

        for source in sources {
            check_pause_and_cancel_simple(tracker)?;
            let source_path = PathBuf::from(source);
            if !source_path.exists() {
                return Err(FileSystemError::new(format!("Source does not exist: {}", source)));
            }

            let file_name = source_path
                .file_name()
                .ok_or_else(|| FileSystemError::new("Could not get file name"))?;
            let dest_file_path = dest_path.join(file_name);

            // Try rename first (same filesystem optimization)
            if fs::rename(&source_path, &dest_file_path).is_err() {
                // If rename fails, copy then delete
                if source_path.is_dir() {
                    copy_dir_real_fs_simple(&source_path, &dest_file_path, tracker)?;
                } else {
                    copy_file_real_fs_simple(&source_path, &dest_file_path, tracker)?;
                }

                // Delete source
                if source_path.is_dir() {
                    fs::remove_dir_all(&source_path)
                        .map_err(|e| FileSystemError::new(format!("Failed to delete source: {}", e)))?;
                } else {
                    fs::remove_file(&source_path)
                        .map_err(|e| FileSystemError::new(format!("Failed to delete source: {}", e)))?;
                }
            }

            tracker.add_item();
        }
    } else {
        // Generic Move: Copy then Delete (supports Virtual & Cross-FS)
        for source in sources {
            // 1. Copy
            copy_recursive_generic(source, destination, tracker, source_fs, dest_fs)?;
            
            // 2. Delete source
            check_pause_and_cancel_simple(tracker)?;
            API.files.delete_item(source, source_fs)
                .map_err(|e| FileSystemError::new(format!("Failed to delete source after move: {}", e)))?;
        }
    }

    Ok(())
}

/// Delete items without AppHandle (for queue operations)
pub fn delete_items_simple(
    paths: &[String],
    tracker: &Arc<ProgressTracker>,
    panel_fs: Option<&str>,
) -> Result<(), FileSystemError> {
    let is_real = panel_fs.is_none() || panel_fs == Some("real");

    if is_real {
        for path_str in paths {
            check_pause_and_cancel_simple(tracker)?;
            let path = PathBuf::from(path_str);

            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .map_err(|e| FileSystemError::new(format!("Failed to delete directory: {}", e)))?;
            } else {
                fs::remove_file(&path)
                    .map_err(|e| FileSystemError::new(format!("Failed to delete file: {}", e)))?;
            }

            tracker.add_item();
        }
    } else {
        // Virtual/Generic Delete
        for path in paths {
            check_pause_and_cancel_simple(tracker)?;
            tracker.update_current_file(Some(path.clone()));
            
            API.files.delete_item(path, panel_fs)
                .map_err(|e| FileSystemError::new(format!("Failed to delete item: {}", e)))?;
            
            tracker.add_item();
        }
    }

    Ok(())
}