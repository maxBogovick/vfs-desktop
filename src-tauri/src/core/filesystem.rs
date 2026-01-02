use serde::{Deserialize, Serialize};
use std::fmt;

/// Результат операции файловой системы
pub type FileSystemResult<T> = Result<T, FileSystemError>;

/// Ошибки файловой системы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemError {
    pub message: String,
}

impl FileSystemError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<String> for FileSystemError {
    fn from(s: String) -> Self {
        FileSystemError::new(s)
    }
}

impl From<&str> for FileSystemError {
    fn from(s: &str) -> Self {
        FileSystemError::new(s)
    }
}

/// Запись файловой системы (файл или директория)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "api-server", derive(utoipa::ToSchema))]
pub struct FileSystemEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub is_file: bool,
    pub size: Option<u64>,
    pub modified: Option<u64>,
    pub created: Option<u64>,
    pub accessed: Option<u64>,
}

/// Trait для различных реализаций файловой системы
/// Может быть реализован для:
/// - Real OS filesystem (fs.json::*)
/// - Virtual filesystem (in-memory)
/// - Database-backed filesystem
pub trait FileSystem: Send + Sync {
    /// Чтение содержимого директории
    fn read_directory(&self, path: &str) -> FileSystemResult<Vec<FileSystemEntry>>;

    /// Получение информации о файле/директории
    fn get_file_info(&self, path: &str) -> FileSystemResult<FileSystemEntry>;

    /// Удаление файла или директории
    fn delete_item(&self, path: &str) -> FileSystemResult<()>;

    /// Переименование файла или директории
    fn rename_item(&self, old_path: &str, new_name: &str) -> FileSystemResult<()>;

    /// Создание новой директории
    fn create_folder(&self, path: &str, name: &str) -> FileSystemResult<()>;

    /// Создание нового файла с опциональным содержимым
    fn create_file(&self, path: &str, name: &str, content: Option<&str>) -> FileSystemResult<()>;

    /// Batch создание файлов
    /// Возвращает вектор результатов для каждого файла
    fn create_files_batch(
        &self,
        path: &str,
        files: &[(String, Option<String>)],
    ) -> FileSystemResult<Vec<FileSystemResult<()>>>;

    /// Копирование файлов/директорий
    fn copy_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()>;

    /// Копирование файла/директории с переименованием
    fn copy_with_custom_name(
        &self,
        source: &str,
        destination_dir: &str,
        new_name: &str,
    ) -> FileSystemResult<()>;

    /// Перемещение файлов/директорий
    fn move_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()>;

    /// Получение домашней директории
    fn get_home_directory(&self) -> FileSystemResult<String>;

    /// Получение системных папок (Documents, Downloads, etc.)
    fn get_system_folders(&self) -> FileSystemResult<Vec<FileSystemEntry>>;

    /// Чтение содержимого файла
    /// Для изображений возвращает base64, для текстовых файлов - строку
    fn read_file_content(&self, path: &str, max_size: Option<u64>) -> FileSystemResult<String>;

    /// Запись содержимого в файл
    /// Перезаписывает существующий файл или создает новый
    fn write_file_content(&self, path: &str, content: &str) -> FileSystemResult<()>;

    /// Открытие файла в системном приложении
    fn open_file(&self, path: &str) -> FileSystemResult<()>;

    /// Показать файл в файловом менеджере ОС
    fn reveal_in_finder(&self, path: &str) -> FileSystemResult<()>;

    /// Нормализация пути (раскрытие ~, резолв к абсолютному пути)
    /// Проверяет существование пути и возвращает канонический абсолютный путь
    fn normalize_path(&self, path: &str) -> FileSystemResult<String>;

    /// Получение подсказок для автодополнения путей
    /// Возвращает список директорий, соответствующих частичному пути
    fn get_path_suggestions(&self, partial_path: &str) -> FileSystemResult<Vec<String>>;

    /// Открытие терминала в указанной директории
    /// Если path - файл, открывает терминал в родительской директории
    fn open_terminal(&self, path: &str) -> FileSystemResult<()>;
}
