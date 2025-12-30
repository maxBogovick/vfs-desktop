use crate::core::{FileSystem, FileSystemEntry, FileSystemError, FileSystemResult};
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Реализация файловой системы через реальную ОС
pub struct RealFileSystem;

impl RealFileSystem {
    pub fn new() -> Self {
        RealFileSystem
    }

    /// Конвертирует путь в FileSystemEntry
    fn path_to_entry(path: &Path) -> FileSystemResult<FileSystemEntry> {
        let metadata = fs::metadata(path)
            .map_err(|e| FileSystemError::new(format!("Failed to read metadata: {}", e)))?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let path_str = path.to_str().unwrap_or("").to_string();

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let created = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let accessed = metadata
            .accessed()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        Ok(FileSystemEntry {
            path: path_str,
            name,
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            size: if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            },
            modified,
            created,
            accessed,
        })
    }

    /// Рекурсивное копирование директории
    fn copy_dir_recursive(src: &Path, dest: &Path) -> FileSystemResult<()> {
        if !src.is_dir() {
            return Err(FileSystemError::new("Source is not a directory"));
        }

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
                Self::copy_dir_recursive(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)
                    .map_err(|e| FileSystemError::new(format!("Failed to copy file: {}", e)))?;
            }
        }

        Ok(())
    }
}

impl FileSystem for RealFileSystem {
    fn read_directory(&self, path: &str) -> FileSystemResult<Vec<FileSystemEntry>> {
        let dir_path = if path.is_empty() || path == "My Computer" || path == "/" {
            dirs::home_dir()
                .ok_or_else(|| FileSystemError::new("Could not find home directory"))?
        } else {
            PathBuf::from(path)
        };

        if !dir_path.exists() {
            return Err(FileSystemError::new(format!(
                "Directory does not exist: {}",
                path
            )));
        }

        if !dir_path.is_dir() {
            return Err(FileSystemError::new(format!(
                "Path is not a directory: {}",
                path
            )));
        }

        let entries = fs::read_dir(&dir_path)
            .map_err(|e| FileSystemError::new(format!("Failed to read directory: {}", e)))?;

        let mut result = Vec::new();

        for entry in entries {
            let entry = entry
                .map_err(|e| FileSystemError::new(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            // Skip hidden files on Unix-like systems
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with('.') {
                        continue;
                    }
                }
            }

            match Self::path_to_entry(&path) {
                Ok(fs_entry) => result.push(fs_entry),
                Err(_) => continue, // Skip files we can't read
            }
        }

        // Sort: directories first, then by name
        result.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(result)
    }

    fn get_file_info(&self, path: &str) -> FileSystemResult<FileSystemEntry> {
        let file_path = PathBuf::from(path);

        if !file_path.exists() {
            return Err(FileSystemError::new(format!(
                "Path does not exist: {}",
                path
            )));
        }

        Self::path_to_entry(&file_path)
    }

    fn delete_item(&self, path: &str) -> FileSystemResult<()> {
        let file_path = PathBuf::from(path);

        if !file_path.exists() {
            return Err(FileSystemError::new(format!(
                "Path does not exist: {}",
                path
            )));
        }

        if file_path.is_dir() {
            fs::remove_dir_all(&file_path)
                .map_err(|e| FileSystemError::new(format!("Failed to delete directory: {}", e)))?;
        } else {
            fs::remove_file(&file_path)
                .map_err(|e| FileSystemError::new(format!("Failed to delete file: {}", e)))?;
        }

        Ok(())
    }

    fn rename_item(&self, old_path: &str, new_name: &str) -> FileSystemResult<()> {
        let old_file_path = PathBuf::from(old_path);

        if !old_file_path.exists() {
            return Err(FileSystemError::new(format!(
                "Path does not exist: {}",
                old_path
            )));
        }

        let parent = old_file_path
            .parent()
            .ok_or_else(|| FileSystemError::new("Could not get parent directory"))?;

        let new_file_path = parent.join(new_name);

        if new_file_path.exists() {
            return Err(FileSystemError::new(format!(
                "File already exists: {}",
                new_name
            )));
        }

        fs::rename(&old_file_path, &new_file_path)
            .map_err(|e| FileSystemError::new(format!("Failed to rename: {}", e)))?;

        Ok(())
    }

    fn create_folder(&self, path: &str, name: &str) -> FileSystemResult<()> {
        let dir_path = PathBuf::from(path);

        if !dir_path.exists() {
            return Err(FileSystemError::new(format!(
                "Parent directory does not exist: {}",
                path
            )));
        }

        let new_folder_path = dir_path.join(name);

        if new_folder_path.exists() {
            return Err(FileSystemError::new(format!(
                "Folder already exists: {}",
                name
            )));
        }

        fs::create_dir(&new_folder_path)
            .map_err(|e| FileSystemError::new(format!("Failed to create folder: {}", e)))?;

        Ok(())
    }

    fn create_file(&self, path: &str, name: &str, content: Option<&str>) -> FileSystemResult<()> {
        use std::io::Write;

        let dir_path = PathBuf::from(path);

        if !dir_path.exists() {
            return Err(FileSystemError::new(format!(
                "Parent directory does not exist: {}",
                path
            )));
        }

        let new_file_path = dir_path.join(name);

        if new_file_path.exists() {
            return Err(FileSystemError::new(format!(
                "File already exists: {}",
                name
            )));
        }

        let mut file = fs::File::create(&new_file_path)
            .map_err(|e| FileSystemError::new(format!("Failed to create file: {}", e)))?;

        if let Some(content_str) = content {
            file.write_all(content_str.as_bytes())
                .map_err(|e| FileSystemError::new(format!("Failed to write file content: {}", e)))?;
        }

        Ok(())
    }

    fn create_files_batch(
        &self,
        path: &str,
        files: &[(String, Option<String>)],
    ) -> FileSystemResult<Vec<FileSystemResult<()>>> {
        let dir_path = PathBuf::from(path);

        if !dir_path.exists() {
            return Err(FileSystemError::new(format!(
                "Parent directory does not exist: {}",
                path
            )));
        }

        let mut results = Vec::new();

        for (name, content) in files {
            let result = self.create_file(path, name, content.as_deref());
            results.push(result);
        }

        Ok(results)
    }

    fn copy_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()> {
        let dest_path = PathBuf::from(destination);

        if !dest_path.exists() || !dest_path.is_dir() {
            return Err(FileSystemError::new(format!(
                "Destination is not a valid directory: {}",
                destination
            )));
        }

        for source in sources {
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
                Self::copy_dir_recursive(&source_path, &dest_file_path)?;
            } else {
                fs::copy(&source_path, &dest_file_path)
                    .map_err(|e| FileSystemError::new(format!("Failed to copy file: {}", e)))?;
            }
        }

        Ok(())
    }

    fn copy_with_custom_name(
        &self,
        source: &str,
        destination_dir: &str,
        new_name: &str,
    ) -> FileSystemResult<()> {
        let source_path = PathBuf::from(source);
        let dest_dir = PathBuf::from(destination_dir);
        let dest_path = dest_dir.join(new_name);

        // Check if source exists
        if !source_path.exists() {
            return Err(FileSystemError::new(format!(
                "Source does not exist: {}",
                source
            )));
        }

        // Check if destination directory exists
        if !dest_dir.exists() || !dest_dir.is_dir() {
            return Err(FileSystemError::new(format!(
                "Destination is not a valid directory: {}",
                destination_dir
            )));
        }

        // Copy file or directory
        if source_path.is_file() {
            fs::copy(&source_path, &dest_path)
                .map_err(|e| FileSystemError::new(format!("Failed to copy file: {}", e)))?;

            // Copy permissions
            if let Ok(metadata) = fs::metadata(&source_path) {
                let _ = fs::set_permissions(&dest_path, metadata.permissions());
            }
        } else if source_path.is_dir() {
            Self::copy_dir_recursive(&source_path, &dest_path)?;
        } else {
            return Err(FileSystemError::new(
                "Source is neither a file nor a directory".to_string(),
            ));
        }

        Ok(())
    }

    fn move_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()> {
        let dest_path = PathBuf::from(destination);

        if !dest_path.exists() || !dest_path.is_dir() {
            return Err(FileSystemError::new(format!(
                "Destination is not a valid directory: {}",
                destination
            )));
        }

        for source in sources {
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

            fs::rename(&source_path, &dest_file_path)
                .map_err(|e| FileSystemError::new(format!("Failed to move: {}", e)))?;
        }

        Ok(())
    }

    fn get_home_directory(&self) -> FileSystemResult<String> {
        dirs::home_dir()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .ok_or_else(|| FileSystemError::new("Could not find home directory"))
    }

    fn get_system_folders(&self) -> FileSystemResult<Vec<FileSystemEntry>> {
        let mut folders = Vec::new();

        // Home directory
        if let Some(home) = dirs::home_dir() {
            if let Ok(entry) = Self::path_to_entry(&home) {
                folders.push(entry);
            }
        }

        // Documents
        if let Some(docs) = dirs::document_dir() {
            if let Ok(entry) = Self::path_to_entry(&docs) {
                folders.push(entry);
            }
        }

        // Downloads
        if let Some(downloads) = dirs::download_dir() {
            if let Ok(entry) = Self::path_to_entry(&downloads) {
                folders.push(entry);
            }
        }

        // Pictures
        if let Some(pictures) = dirs::picture_dir() {
            if let Ok(entry) = Self::path_to_entry(&pictures) {
                folders.push(entry);
            }
        }

        // Music
        if let Some(music) = dirs::audio_dir() {
            if let Ok(entry) = Self::path_to_entry(&music) {
                folders.push(entry);
            }
        }

        // Videos
        if let Some(videos) = dirs::video_dir() {
            if let Ok(entry) = Self::path_to_entry(&videos) {
                folders.push(entry);
            }
        }

        // Desktop
        if let Some(desktop) = dirs::desktop_dir() {
            if let Ok(entry) = Self::path_to_entry(&desktop) {
                folders.push(entry);
            }
        }

        Ok(folders)
    }

    fn read_file_content(&self, path: &str, max_size: Option<u64>) -> FileSystemResult<String> {
        let file_path = PathBuf::from(path);

        if !file_path.exists() {
            return Err(FileSystemError::new(format!(
                "File does not exist: {}",
                path
            )));
        }

        if !file_path.is_file() {
            return Err(FileSystemError::new(format!(
                "Path is not a file: {}",
                path
            )));
        }

        let metadata = fs::metadata(&file_path)
            .map_err(|e| FileSystemError::new(format!("Failed to read metadata: {}", e)))?;

        let file_size = metadata.len();
        let limit = max_size.unwrap_or(10_000_000); // Default 10MB limit

        if file_size > limit {
            return Err(FileSystemError::new(format!(
                "File too large: {} bytes (limit: {} bytes)",
                file_size, limit
            )));
        }

        // Check file extension to determine if it's binary or text
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let image_extensions = vec!["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"];

        if image_extensions.contains(&extension.as_str()) {
            // For images, return base64 encoded content
            let mut file = fs::File::open(&file_path)
                .map_err(|e| FileSystemError::new(format!("Failed to open file: {}", e)))?;

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| FileSystemError::new(format!("Failed to read file: {}", e)))?;

            Ok(general_purpose::STANDARD.encode(&buffer))
        } else {
            // For text files, return content as string
            fs::read_to_string(&file_path)
                .map_err(|e| FileSystemError::new(format!("Failed to read file as text: {}", e)))
        }
    }

    fn open_file(&self, path: &str) -> FileSystemResult<()> {
        let file_path = PathBuf::from(path);

        if !file_path.exists() {
            return Err(FileSystemError::new(format!(
                "File does not exist: {}",
                path
            )));
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(&file_path)
                .spawn()
                .map_err(|e| FileSystemError::new(format!("Failed to open file: {}", e)))?;
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(&["/C", "start", "", path])
                .spawn()
                .map_err(|e| FileSystemError::new(format!("Failed to open file: {}", e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(&file_path)
                .spawn()
                .map_err(|e| FileSystemError::new(format!("Failed to open file: {}", e)))?;
        }

        Ok(())
    }

    fn reveal_in_finder(&self, path: &str) -> FileSystemResult<()> {
        let file_path = PathBuf::from(path);

        if !file_path.exists() {
            return Err(FileSystemError::new(format!(
                "Path does not exist: {}",
                path
            )));
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .args(&["-R", file_path.to_str().unwrap()])
                .spawn()
                .map_err(|e| FileSystemError::new(format!("Failed to reveal in Finder: {}", e)))?;
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .args(&["/select,", path])
                .spawn()
                .map_err(|e| FileSystemError::new(format!("Failed to reveal in Explorer: {}", e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            // Just open the parent directory
            if let Some(parent) = file_path.parent() {
                Command::new("xdg-open")
                    .arg(parent)
                    .spawn()
                    .map_err(|e| FileSystemError::new(format!("Failed to open directory: {}", e)))?;
            }
        }

        Ok(())
    }

    fn normalize_path(&self, path: &str) -> FileSystemResult<String> {
        // Раскрываем тильду если есть
        let expanded = if path.starts_with("~/") || path == "~" {
            let home = dirs::home_dir()
                .ok_or_else(|| FileSystemError::new("Could not find home directory"))?;

            if path == "~" {
                home
            } else {
                home.join(&path[2..])
            }
        } else {
            PathBuf::from(path)
        };

        // Проверяем существование пути
        if !expanded.exists() {
            return Err(FileSystemError::new(format!(
                "Path does not exist: {}",
                path
            )));
        }

        // Возвращаем абсолютный путь
        expanded
            .canonicalize()
            .map_err(|e| FileSystemError::new(format!("Failed to normalize path: {}", e)))?
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| FileSystemError::new("Invalid path encoding"))
    }

    fn get_path_suggestions(&self, partial_path: &str) -> FileSystemResult<Vec<String>> {
        // Раскрываем тильду если есть
        let expanded = if partial_path.starts_with("~/") || partial_path == "~" {
            let home = dirs::home_dir()
                .ok_or_else(|| FileSystemError::new("Could not find home directory"))?;

            if partial_path == "~" {
                home.to_str().unwrap_or("").to_string()
            } else {
                home.join(&partial_path[2..])
                    .to_str()
                    .unwrap_or("")
                    .to_string()
            }
        } else {
            partial_path.to_string()
        };

        let path = PathBuf::from(&expanded);

        // Определяем родительскую директорию и префикс для поиска
        let (parent_dir, prefix) = if expanded.ends_with('/') || expanded.ends_with('\\') {
            // Если путь заканчивается на /, ищем все в этой директории
            (path.clone(), String::new())
        } else {
            // Иначе берем родительскую директорию и имя файла как префикс
            let parent = path.parent().unwrap_or_else(|| Path::new("/"));
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            (parent.to_path_buf(), file_name)
        };

        // Если родительская директория не существует, возвращаем пустой список
        if !parent_dir.exists() || !parent_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut suggestions = Vec::new();

        // Читаем содержимое директории
        let entries = fs::read_dir(&parent_dir)
            .map_err(|e| FileSystemError::new(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let file_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Пропускаем скрытые файлы
                if file_name.starts_with('.') {
                    continue;
                }

                // Фильтруем по префиксу (без учета регистра)
                if !prefix.is_empty()
                    && !file_name.to_lowercase().starts_with(&prefix.to_lowercase())
                {
                    continue;
                }

                // Добавляем только директории для автодополнения путей
                if entry_path.is_dir() {
                    if let Some(path_str) = entry_path.to_str() {
                        // Преобразуем обратно в формат с ~ если начинается с home
                        let suggestion = if partial_path.starts_with("~") {
                            if let Some(home) = dirs::home_dir() {
                                if let Some(home_str) = home.to_str() {
                                    path_str
                                        .strip_prefix(home_str)
                                        .map(|suffix| format!("~{}", suffix))
                                        .unwrap_or_else(|| path_str.to_string())
                                } else {
                                    path_str.to_string()
                                }
                            } else {
                                path_str.to_string()
                            }
                        } else {
                            path_str.to_string()
                        };

                        suggestions.push(suggestion);
                    }
                }
            }
        }

        // Сортируем по имени
        suggestions.sort();

        // Ограничиваем количество подсказок
        suggestions.truncate(10);

        Ok(suggestions)
    }

    fn open_terminal(&self, path: &str) -> FileSystemResult<()> {
        let path = Path::new(path);

        // Определяем директорию для открытия
        let dir = if path.is_file() {
            path.parent().ok_or_else(||
                FileSystemError::new("Could not determine parent directory")
            )?
        } else if path.is_dir() {
            path
        } else {
            return Err(FileSystemError::new(
                format!("Path does not exist: {}", path.display())
            ));
        };

        // Открываем терминал в зависимости от ОС
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg("-a")
                .arg("Terminal")
                .arg(dir)
                .spawn()
                .map_err(|e| FileSystemError::new(
                    format!("Failed to open terminal: {}", e)
                ))?;
        }

        #[cfg(target_os = "linux")]
        {
            // Пробуем разные терминалы в порядке приоритета
            let terminals = vec![
                ("gnome-terminal", vec!["--working-directory"]),
                ("konsole", vec!["--workdir"]),
                ("xfce4-terminal", vec!["--working-directory"]),
                ("x-terminal-emulator", vec!["-e", "cd"]),
            ];

            let mut success = false;
            for (terminal, args) in terminals {
                let mut cmd = Command::new(terminal);
                for arg in args {
                    cmd.arg(arg);
                }
                cmd.arg(dir);

                if cmd.spawn().is_ok() {
                    success = true;
                    break;
                }
            }

            if !success {
                return Err(FileSystemError::new(
                    "No terminal emulator found"
                ));
            }
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .arg("/c")
                .arg("start")
                .arg("cmd")
                .arg("/K")
                .arg("cd")
                .arg("/D")
                .arg(dir)
                .spawn()
                .map_err(|e| FileSystemError::new(
                    format!("Failed to open terminal: {}", e)
                ))?;
        }

        Ok(())
    }
}
