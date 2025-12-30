/**
 * Async File Operations for REST API
 *
 * Упрощенные async обертки для file operations с WebSocket progress updates.
 * Используют стандартные fs операции с прогресс-трекингом на уровне файлов.
 */

#[cfg(feature = "api-server")]
use crate::api_server::state::AppState;
use crate::core::FileSystemError;
use crate::progress::{ProgressTracker, emit_progress_websocket};
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;

/// Копирует элементы с прогрессом (async версия для REST API)
#[cfg(feature = "api-server")]
pub async fn copy_items_with_progress_async(
    sources: Vec<String>,
    destination: String,
    tracker: Arc<ProgressTracker>,
    state: Arc<AppState>,
) -> Result<(), FileSystemError> {
    tokio::task::spawn_blocking(move || {
        use crate::api::RealFileSystem;
        use crate::core::FileSystem;

        let fs = RealFileSystem::new();
        let mut completed_items = 0u64;

        for source in &sources {
            // Проверяем отмену
            if tracker.is_cancelled() {
                tracker.mark_cancelled();
                emit_progress_websocket(&state, &tracker);
                return Err(FileSystemError::new("Operation cancelled"));
            }

            // Обновляем текущий файл
            let file_name = PathBuf::from(source)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(source)
                .to_string();

            tracker.update_current_file(Some(file_name));
            emit_progress_websocket(&state, &tracker);

            // Копируем через FileSystem API
            if let Err(e) = fs.copy_items(&[source.clone()], &destination) {
                tracker.mark_failed(e.message.clone());
                emit_progress_websocket(&state, &tracker);
                return Err(FileSystemError::new(e.message));
            }

            completed_items += 1;
            tracker.add_item();
            emit_progress_websocket(&state, &tracker);
        }

        tracker.mark_completed();
        emit_progress_websocket(&state, &tracker);
        Ok(())
    })
    .await
    .map_err(|e| FileSystemError::new(format!("Task join error: {}", e)))?
}

/// Перемещает элементы с прогрессом (async версия для REST API)
#[cfg(feature = "api-server")]
pub async fn move_items_with_progress_async(
    sources: Vec<String>,
    destination: String,
    tracker: Arc<ProgressTracker>,
    state: Arc<AppState>,
) -> Result<(), FileSystemError> {
    tokio::task::spawn_blocking(move || {
        use crate::api::RealFileSystem;
        use crate::core::FileSystem;

        let fs = RealFileSystem::new();

        for source in &sources {
            if tracker.is_cancelled() {
                tracker.mark_cancelled();
                emit_progress_websocket(&state, &tracker);
                return Err(FileSystemError::new("Operation cancelled"));
            }

            let file_name = PathBuf::from(source)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(source)
                .to_string();

            tracker.update_current_file(Some(file_name));
            emit_progress_websocket(&state, &tracker);

            if let Err(e) = fs.move_items(&[source.clone()], &destination) {
                tracker.mark_failed(e.message.clone());
                emit_progress_websocket(&state, &tracker);
                return Err(FileSystemError::new(e.message));
            }

            tracker.add_item();
            emit_progress_websocket(&state, &tracker);
        }

        tracker.mark_completed();
        emit_progress_websocket(&state, &tracker);
        Ok(())
    })
    .await
    .map_err(|e| FileSystemError::new(format!("Task join error: {}", e)))?
}

/// Удаляет элементы с прогрессом (async версия для REST API)
#[cfg(feature = "api-server")]
pub async fn delete_items_with_progress_async(
    paths: Vec<String>,
    tracker: Arc<ProgressTracker>,
    state: Arc<AppState>,
) -> Result<(), FileSystemError> {
    tokio::task::spawn_blocking(move || {
        use crate::api::RealFileSystem;
        use crate::core::FileSystem;

        let fs = RealFileSystem::new();

        for path in &paths {
            if tracker.is_cancelled() {
                tracker.mark_cancelled();
                emit_progress_websocket(&state, &tracker);
                return Err(FileSystemError::new("Operation cancelled"));
            }

            let file_name = PathBuf::from(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path)
                .to_string();

            tracker.update_current_file(Some(file_name));
            emit_progress_websocket(&state, &tracker);

            if let Err(e) = fs.delete_item(path) {
                tracker.mark_failed(e.message.clone());
                emit_progress_websocket(&state, &tracker);
                return Err(FileSystemError::new(e.message));
            }

            tracker.add_item();
            emit_progress_websocket(&state, &tracker);
        }

        tracker.mark_completed();
        emit_progress_websocket(&state, &tracker);
        Ok(())
    })
    .await
    .map_err(|e| FileSystemError::new(format!("Task join error: {}", e)))?
}
