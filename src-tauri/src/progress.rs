use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tauri::Emitter;
use once_cell::sync::Lazy;

/// Тип операции
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OperationType {
    Copy,
    Move,
    Delete,
}

/// Состояние операции
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    Running,
    Paused,
    Completed,
    Cancelled,
    Failed,
}

/// Событие прогресса операции
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub operation_id: String,
    pub operation_type: OperationType,
    pub status: OperationStatus,
    pub current_bytes: u64,
    pub total_bytes: u64,
    pub current_items: u64,
    pub total_items: u64,
    pub current_file: Option<String>,
    pub speed_bytes_per_sec: f64,
    pub eta_seconds: Option<f64>,
    pub error_message: Option<String>,
}

/// Токен отмены и паузы операции
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
}

/// Трекер прогресса операции
pub struct ProgressTracker {
    operation_id: String,
    operation_type: OperationType,
    current_bytes: Arc<AtomicU64>,
    total_bytes: Arc<AtomicU64>,
    current_items: Arc<AtomicU64>,
    total_items: Arc<AtomicU64>,
    current_file: Arc<RwLock<Option<String>>>,
    start_time: Instant,
    last_update: Arc<RwLock<Instant>>,
    bytes_at_last_update: Arc<AtomicU64>,
    cancellation_token: CancellationToken,
    status: Arc<RwLock<OperationStatus>>,
    error_message: Arc<RwLock<Option<String>>>,
}

impl ProgressTracker {
    pub fn new(
        operation_id: String,
        operation_type: OperationType,
        total_bytes: u64,
        total_items: u64,
    ) -> Self {
        Self {
            operation_id,
            operation_type,
            current_bytes: Arc::new(AtomicU64::new(0)),
            total_bytes: Arc::new(AtomicU64::new(total_bytes)),
            current_items: Arc::new(AtomicU64::new(0)),
            total_items: Arc::new(AtomicU64::new(total_items)),
            current_file: Arc::new(RwLock::new(None)),
            start_time: Instant::now(),
            last_update: Arc::new(RwLock::new(Instant::now())),
            bytes_at_last_update: Arc::new(AtomicU64::new(0)),
            cancellation_token: CancellationToken::new(),
            status: Arc::new(RwLock::new(OperationStatus::Running)),
            error_message: Arc::new(RwLock::new(None)),
        }
    }

    pub fn update_current_file(&self, file_path: Option<String>) {
        let mut current = self.current_file.write().unwrap();
        *current = file_path;
    }

    pub fn add_bytes(&self, bytes: u64) {
        self.current_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn add_item(&self) {
        self.current_items.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_total_bytes(&self, bytes: u64) {
        self.total_bytes.store(bytes, Ordering::Relaxed);
    }

    pub fn get_current_bytes(&self) -> u64 {
        self.current_bytes.load(Ordering::Relaxed)
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    pub fn is_paused(&self) -> bool {
        self.cancellation_token.is_paused()
    }

    pub fn pause(&self) {
        self.cancellation_token.pause();
        let mut status = self.status.write().unwrap();
        *status = OperationStatus::Paused;
    }

    pub fn resume(&self) {
        self.cancellation_token.resume();
        let mut status = self.status.write().unwrap();
        *status = OperationStatus::Running;
    }

    pub fn mark_completed(&self) {
        let mut status = self.status.write().unwrap();
        *status = OperationStatus::Completed;
    }

    pub fn mark_cancelled(&self) {
        let mut status = self.status.write().unwrap();
        *status = OperationStatus::Cancelled;
    }

    pub fn mark_failed(&self, error: String) {
        let mut status = self.status.write().unwrap();
        *status = OperationStatus::Failed;

        let mut error_msg = self.error_message.write().unwrap();
        *error_msg = Some(error);
    }

    pub fn get_progress_event(&self) -> ProgressEvent {
        let current_bytes = self.current_bytes.load(Ordering::Relaxed);
        let total_bytes = self.total_bytes.load(Ordering::Relaxed);
        let current_items = self.current_items.load(Ordering::Relaxed);
        let total_items = self.total_items.load(Ordering::Relaxed);

        let current_file = self.current_file.read().unwrap().clone();
        let status = self.status.read().unwrap().clone();
        let error_message = self.error_message.read().unwrap().clone();

        // Вычисляем скорость
        let elapsed = self.start_time.elapsed();
        let speed_bytes_per_sec = if elapsed.as_secs() > 0 {
            current_bytes as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        // Вычисляем ETA
        let eta_seconds = if speed_bytes_per_sec > 0.0 && total_bytes > current_bytes {
            let remaining_bytes = total_bytes - current_bytes;
            Some(remaining_bytes as f64 / speed_bytes_per_sec)
        } else {
            None
        };

        ProgressEvent {
            operation_id: self.operation_id.clone(),
            operation_type: self.operation_type.clone(),
            status,
            current_bytes,
            total_bytes,
            current_items,
            total_items,
            current_file,
            speed_bytes_per_sec,
            eta_seconds,
            error_message,
        }
    }

    pub fn should_emit_update(&self) -> bool {
        let now = Instant::now();
        let last_update = *self.last_update.read().unwrap();

        // Отправляем обновление максимум раз в 100ms
        if now.duration_since(last_update) > Duration::from_millis(100) {
            let mut last = self.last_update.write().unwrap();
            *last = now;

            let current_bytes = self.current_bytes.load(Ordering::Relaxed);
            self.bytes_at_last_update.store(current_bytes, Ordering::Relaxed);

            true
        } else {
            false
        }
    }
}

/// Глобальный менеджер операций
pub struct OperationsManager {
    operations: Arc<RwLock<HashMap<String, Arc<ProgressTracker>>>>,
}

impl OperationsManager {
    pub fn new() -> Self {
        Self {
            operations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_operation(
        &self,
        operation_id: String,
        operation_type: OperationType,
        total_bytes: u64,
        total_items: u64,
    ) -> Arc<ProgressTracker> {
        let tracker = Arc::new(ProgressTracker::new(
            operation_id.clone(),
            operation_type,
            total_bytes,
            total_items,
        ));

        let mut operations = self.operations.write().unwrap();
        operations.insert(operation_id, tracker.clone());

        tracker
    }

    pub fn get_operation(&self, operation_id: &str) -> Option<Arc<ProgressTracker>> {
        let operations = self.operations.read().unwrap();
        operations.get(operation_id).cloned()
    }

    pub fn cancel_operation(&self, operation_id: &str) -> bool {
        if let Some(tracker) = self.get_operation(operation_id) {
            tracker.cancellation_token().cancel();
            tracker.mark_cancelled();
            true
        } else {
            false
        }
    }

    pub fn pause_operation(&self, operation_id: &str) -> bool {
        if let Some(tracker) = self.get_operation(operation_id) {
            tracker.pause();
            true
        } else {
            false
        }
    }

    pub fn resume_operation(&self, operation_id: &str) -> bool {
        if let Some(tracker) = self.get_operation(operation_id) {
            tracker.resume();
            true
        } else {
            false
        }
    }

    pub fn remove_operation(&self, operation_id: &str) {
        let mut operations = self.operations.write().unwrap();
        operations.remove(operation_id);
    }

    pub fn get_all_operations(&self) -> Vec<Arc<ProgressTracker>> {
        let operations = self.operations.read().unwrap();
        operations.values().cloned().collect()
    }
}

/// Глобальный экземпляр менеджера операций
pub static OPERATIONS_MANAGER: Lazy<OperationsManager> = Lazy::new(|| OperationsManager::new());

/// Emit прогресс события в frontend
pub fn emit_progress<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    tracker: &Arc<ProgressTracker>,
) {
    if tracker.should_emit_update() || tracker.is_cancelled() {
        let event = tracker.get_progress_event();
        let _ = app.emit("file-operation-progress", event);
    }
}
