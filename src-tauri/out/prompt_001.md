# 📦 Code Context - Part 1/1

> Generated: 2026-01-11 20:36:35
> Files in this chunk: 35
> Total tokens: ~13556

---


## 📄 `src-tauri/build.rs`

**Tokens:** ~7 | **Lines:** 3
```rust
fn main() {
    tauri_build::build()
}
```


---


## 📄 `src-tauri/src/api/mod.rs`

**Tokens:** ~14 | **Lines:** 3
```rust
pub mod real_fs;
pub mod virtual_fs;
pub use real_fs::RealFileSystem;
```


---


## 📄 `src-tauri/src/api/real_fs.rs`

**Tokens:** ~4011 | **Lines:** 563
```rust
use crate::core::{FileSystem, FileSystemEntry, FileSystemError, FileSystemResult};
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
pub struct RealFileSystem;
impl RealFileSystem {
    pub fn new() -> Self {
        RealFileSystem
    }
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
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with('.') {
                        continue;
                    }
                }
            }
            match Self::path_to_entry(&path) {
                Ok(fs_entry) => result.push(fs_entry),
                Err(_) => continue,
            }
        }
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
        if let Some(home) = dirs::home_dir() {
            if let Ok(entry) = Self::path_to_entry(&home) {
                folders.push(entry);
            }
        }
        if let Some(docs) = dirs::document_dir() {
            if let Ok(entry) = Self::path_to_entry(&docs) {
                folders.push(entry);
            }
        }
        if let Some(downloads) = dirs::download_dir() {
            if let Ok(entry) = Self::path_to_entry(&downloads) {
                folders.push(entry);
            }
        }
        if let Some(pictures) = dirs::picture_dir() {
            if let Ok(entry) = Self::path_to_entry(&pictures) {
                folders.push(entry);
            }
        }
        if let Some(music) = dirs::audio_dir() {
            if let Ok(entry) = Self::path_to_entry(&music) {
                folders.push(entry);
            }
        }
        if let Some(videos) = dirs::video_dir() {
            if let Ok(entry) = Self::path_to_entry(&videos) {
                folders.push(entry);
            }
        }
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
        let limit = max_size.unwrap_or(10_000_000);
        if file_size > limit {
            return Err(FileSystemError::new(format!(
                "File too large: {} bytes (limit: {} bytes)",
                file_size, limit
            )));
        }
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let image_extensions = vec!["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"];
        if image_extensions.contains(&extension.as_str()) {
            let mut file = fs::File::open(&file_path)
                .map_err(|e| FileSystemError::new(format!("Failed to open file: {}", e)))?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| FileSystemError::new(format!("Failed to read file: {}", e)))?;
            Ok(general_purpose::STANDARD.encode(&buffer))
        } else {
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
        if !expanded.exists() {
            return Err(FileSystemError::new(format!(
                "Path does not exist: {}",
                path
            )));
        }
        expanded
            .canonicalize()
            .map_err(|e| FileSystemError::new(format!("Failed to normalize path: {}", e)))?
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| FileSystemError::new("Invalid path encoding"))
    }
    fn get_path_suggestions(&self, partial_path: &str) -> FileSystemResult<Vec<String>> {
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
        let (parent_dir, prefix) = if expanded.ends_with('/') || expanded.ends_with('\\') {
            (path.clone(), String::new())
        } else {
            let parent = path.parent().unwrap_or_else(|| Path::new("/"));
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            (parent.to_path_buf(), file_name)
        };
        if !parent_dir.exists() || !parent_dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut suggestions = Vec::new();
        let entries = fs::read_dir(&parent_dir)
            .map_err(|e| FileSystemError::new(format!("Failed to read directory: {}", e)))?;
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let file_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if file_name.starts_with('.') {
                    continue;
                }
                if !prefix.is_empty()
                    && !file_name.to_lowercase().starts_with(&prefix.to_lowercase())
                {
                    continue;
                }
                if entry_path.is_dir() {
                    if let Some(path_str) = entry_path.to_str() {
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
        suggestions.sort();
        suggestions.truncate(10);
        Ok(suggestions)
    }
    fn open_terminal(&self, path: &str) -> FileSystemResult<()> {
        let path = Path::new(path);
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
```


---


## 📄 `src-tauri/src/api/virtual_fs.rs`

**Tokens:** ~3822 | **Lines:** 487
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::core::{FileSystem, FileSystemEntry, FileSystemError, FileSystemResult};
#[derive(Debug, Clone, Serialize, Deserialize)]
enum VfsNode {
    File {
        content: Vec<u8>,
        created: u64,
        modified: u64,
    },
    Directory {
        children: HashMap<String, VfsNode>,
        created: u64,
        modified: u64,
    },
}
impl VfsNode {
    fn new_file(content: Vec<u8>) -> Self {
        let now = current_timestamp();
        VfsNode::File {
            content,
            created: now,
            modified: now,
        }
    }
    fn new_directory() -> Self {
        let now = current_timestamp();
        VfsNode::Directory {
            children: HashMap::new(),
            created: now,
            modified: now,
        }
    }
    fn is_dir(&self) -> bool {
        matches!(self, VfsNode::Directory { .. })
    }
    fn size(&self) -> u64 {
        match self {
            VfsNode::File { content, .. } => content.len() as u64,
            VfsNode::Directory { .. } => 0,
        }
    }
    fn created(&self) -> u64 {
        match self {
            VfsNode::File { created, .. } => *created,
            VfsNode::Directory { created, .. } => *created,
        }
    }
    fn modified(&self) -> u64 {
        match self {
            VfsNode::File { modified, .. } => *modified,
            VfsNode::Directory { modified, .. } => *modified,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VfsState {
    root: VfsNode,
    home_directory: String,
}
impl Default for VfsState {
    fn default() -> Self {
        let mut root = VfsNode::new_directory();
        if let VfsNode::Directory { children, .. } = &mut root {
            let mut home = VfsNode::new_directory();
            if let VfsNode::Directory { children: home_children, .. } = &mut home {
                home_children.insert("Documents".to_string(), VfsNode::new_directory());
                home_children.insert("Downloads".to_string(), VfsNode::new_directory());
                home_children.insert("Pictures".to_string(), VfsNode::new_directory());
                home_children.insert("Desktop".to_string(), VfsNode::new_directory());
            }
            children.insert("home".to_string(), home);
        }
        Self {
            root,
            home_directory: "/home".to_string(),
        }
    }
}
pub struct VirtualFileSystem {
    state: Arc<RwLock<VfsState>>,
    persistence_path: PathBuf,
}
impl VirtualFileSystem {
    pub fn new(persistence_path: impl AsRef<Path>) -> FileSystemResult<Self> {
        let persistence_path = persistence_path.as_ref().to_path_buf();
        let state = if persistence_path.exists() {
            Self::load_state(&persistence_path).unwrap_or_else(|e| {
                VfsState::default()
            })
        } else {
            VfsState::default()
        };
        let vfs = Self {
            state: Arc::new(RwLock::new(state)),
            persistence_path,
        };
        vfs.save_state()?;
        Ok(vfs)
    }
    fn load_state(path: &Path) -> FileSystemResult<VfsState> {
        let data = std::fs::read(path)
            .map_err(|e| FileSystemError::new(format!("Не удалось прочитать файл состояния: {}", e)))?;
        if data.is_empty() {
            return Err(FileSystemError::new("Файл состояния пустой"));
        }
        let state: VfsState = serde_json::from_slice(&data)
            .map_err(|e| FileSystemError::new(format!("Не удалось десериализовать состояние: {}", e)))?;
        Ok(state)
    }
    fn save_state(&self) -> FileSystemResult<()> {
        let state = self.state.read()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку для чтения"))?;
        let data = serde_json::to_vec_pretty(&*state)
            .map_err(|e| FileSystemError::new(format!("Не удалось сериализовать состояние: {}", e)))?;
        std::fs::write(&self.persistence_path, data)
            .map_err(|e| FileSystemError::new(format!("Не удалось записать файл состояния: {}", e)))?;
        Ok(())
    }
    fn normalize_path_internal(&self, path: &str) -> String {
        let path = if path.starts_with('~') {
            let state = self.state.read().unwrap();
            path.replacen('~', &state.home_directory, 1)
        } else {
            path.to_string()
        };
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parts.join("/"))
        }
    }
    fn find_node(&self, path: &str) -> FileSystemResult<VfsNode> {
        let normalized = self.normalize_path_internal(path);
        let state = self.state.read()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку"))?;
        if normalized == "/" {
            return Ok(state.root.clone());
        }
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &state.root;
        for part in parts {
            match current {
                VfsNode::Directory { children, .. } => {
                    current = children.get(part)
                        .ok_or_else(|| FileSystemError::new(format!("Путь не найден: {}", path)))?;
                }
                VfsNode::File { .. } => {
                    return Err(FileSystemError::new(format!("'{}' не является директорией", path)));
                }
            }
        }
        Ok(current.clone())
    }
    fn find_parent_and_name(&self, path: &str) -> FileSystemResult<(Vec<String>, String)> {
        let normalized = self.normalize_path_internal(path);
        let parts: Vec<String> = normalized.split('/').filter(|s| !s.is_empty()).map(String::from).collect();
        if parts.is_empty() {
            return Err(FileSystemError::new("Невозможно получить родителя корневой директории"));
        }
        let name = parts.last().unwrap().clone();
        let parent_parts = parts[..parts.len() - 1].to_vec();
        Ok((parent_parts, name))
    }
    fn with_node_mut<F, R>(&self, path: &str, f: F) -> FileSystemResult<R>
    where
        F: FnOnce(&mut VfsNode) -> FileSystemResult<R>,
    {
        let normalized = self.normalize_path_internal(path);
        let mut state = self.state.write()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку для записи"))?;
        if normalized == "/" {
            let result = f(&mut state.root)?;
            drop(state);
            self.save_state()?;
            return Ok(result);
        }
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut state.root;
        for part in parts {
            match current {
                VfsNode::Directory { children, .. } => {
                    current = children.get_mut(part)
                        .ok_or_else(|| FileSystemError::new(format!("Путь не найден: {}", path)))?;
                }
                VfsNode::File { .. } => {
                    return Err(FileSystemError::new(format!("'{}' не является директорией", path)));
                }
            }
        }
        let result = f(current)?;
        drop(state);
        self.save_state()?;
        Ok(result)
    }
}
impl FileSystem for VirtualFileSystem {
    fn read_directory(&self, path: &str) -> FileSystemResult<Vec<FileSystemEntry>> {
        let normalized = self.normalize_path_internal(path);
        let node = self.find_node(&normalized)?;
        match node {
            VfsNode::Directory { children, .. } => {
                let mut entries = Vec::new();
                for (name, child) in children.iter() {
                    let entry_path = if normalized == "/" {
                        format!("/{}", name)
                    } else {
                        format!("{}/{}", normalized, name)
                    };
                    entries.push(FileSystemEntry {
                        path: entry_path,
                        name: name.clone(),
                        is_dir: child.is_dir(),
                        is_file: !child.is_dir(),
                        size: Some(child.size()),
                        modified: Some(child.modified()),
                        created: Some(child.created()),
                        accessed: Some(child.modified()),
                    });
                }
                Ok(entries)
            }
            VfsNode::File { .. } => {
                Err(FileSystemError::new(format!("'{}' не является директорией", path)))
            }
        }
    }
    fn get_file_info(&self, path: &str) -> FileSystemResult<FileSystemEntry> {
        let normalized = self.normalize_path_internal(path);
        let node = self.find_node(&normalized)?;
        let name = normalized.split('/').filter(|s| !s.is_empty()).last()
            .unwrap_or("root").to_string();
        Ok(FileSystemEntry {
            path: normalized.clone(),
            name,
            is_dir: node.is_dir(),
            is_file: !node.is_dir(),
            size: Some(node.size()),
            modified: Some(node.modified()),
            created: Some(node.created()),
            accessed: Some(node.modified()),
        })
    }
    fn delete_item(&self, path: &str) -> FileSystemResult<()> {
        let (parent_parts, name) = self.find_parent_and_name(path)?;
        let mut state = self.state.write()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку"))?;
        let mut current = &mut state.root;
        for part in &parent_parts {
            match current {
                VfsNode::Directory { children, .. } => {
                    current = children.get_mut(part)
                        .ok_or_else(|| FileSystemError::new("Родительская директория не найдена"))?;
                }
                VfsNode::File { .. } => {
                    return Err(FileSystemError::new("Путь содержит файл вместо директории"));
                }
            }
        }
        match current {
            VfsNode::Directory { children, .. } => {
                children.remove(&name)
                    .ok_or_else(|| FileSystemError::new(format!("Элемент '{}' не найден", name)))?;
            }
            VfsNode::File { .. } => {
                return Err(FileSystemError::new("Родитель не является директорией"));
            }
        }
        drop(state);
        self.save_state()?;
        Ok(())
    }
    fn rename_item(&self, old_path: &str, new_name: &str) -> FileSystemResult<()> {
        if new_name.contains('/') {
            return Err(FileSystemError::new("Новое имя не должно содержать '/'"));
        }
        let (parent_parts, old_name) = self.find_parent_and_name(old_path)?;
        let mut state = self.state.write()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку"))?;
        let mut current = &mut state.root;
        for part in &parent_parts {
            match current {
                VfsNode::Directory { children, .. } => {
                    current = children.get_mut(part)
                        .ok_or_else(|| FileSystemError::new("Родительская директория не найдена"))?;
                }
                VfsNode::File { .. } => {
                    return Err(FileSystemError::new("Путь содержит файл"));
                }
            }
        }
        match current {
            VfsNode::Directory { children, .. } => {
                if children.contains_key(new_name) {
                    return Err(FileSystemError::new(format!("Элемент '{}' уже существует", new_name)));
                }
                let node = children.remove(&old_name)
                    .ok_or_else(|| FileSystemError::new(format!("Элемент '{}' не найден", old_name)))?;
                children.insert(new_name.to_string(), node);
            }
            VfsNode::File { .. } => {
                return Err(FileSystemError::new("Родитель не является директорией"));
            }
        }
        drop(state);
        self.save_state()?;
        Ok(())
    }
    fn create_folder(&self, path: &str, name: &str) -> FileSystemResult<()> {
        if name.contains('/') {
            return Err(FileSystemError::new("Имя папки не должно содержать '/'"));
        }
        let normalized = self.normalize_path_internal(path);
        let mut state = self.state.write()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку"))?;
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut state.root;
        for part in parts {
            match current {
                VfsNode::Directory { children, .. } => {
                    current = children.get_mut(part)
                        .ok_or_else(|| FileSystemError::new(format!("Путь не найден: {}", path)))?;
                }
                VfsNode::File { .. } => {
                    return Err(FileSystemError::new("Путь содержит файл"));
                }
            }
        }
        match current {
            VfsNode::Directory { children, modified, .. } => {
                if children.contains_key(name) {
                    return Err(FileSystemError::new(format!("Папка '{}' уже существует", name)));
                }
                children.insert(name.to_string(), VfsNode::new_directory());
                *modified = current_timestamp();
            }
            VfsNode::File { .. } => {
                return Err(FileSystemError::new("Не является директорией"));
            }
        }
        drop(state);
        self.save_state()?;
        Ok(())
    }
    fn copy_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()> {
        let nodes_to_copy: Vec<(String, VfsNode)> = sources.iter()
            .map(|src| {
                let name = src.split('/').filter(|s| !s.is_empty()).last()
                    .ok_or_else(|| FileSystemError::new("Некорректный путь источника"))?
                    .to_string();
                let node = self.find_node(src)?;
                Ok((name, node))
            })
            .collect::<FileSystemResult<Vec<_>>>()?;
        let normalized_dest = self.normalize_path_internal(destination);
        let mut state = self.state.write()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку"))?;
        let parts: Vec<&str> = normalized_dest.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut state.root;
        for part in parts {
            match current {
                VfsNode::Directory { children, .. } => {
                    current = children.get_mut(part)
                        .ok_or_else(|| FileSystemError::new("Директория назначения не найдена"))?;
                }
                VfsNode::File { .. } => {
                    return Err(FileSystemError::new("Назначение не является директорией"));
                }
            }
        }
        match current {
            VfsNode::Directory { children, modified, .. } => {
                for (name, node) in nodes_to_copy {
                    children.insert(name, node);
                }
                *modified = current_timestamp();
            }
            VfsNode::File { .. } => {
                return Err(FileSystemError::new("Назначение не является директорией"));
            }
        }
        drop(state);
        self.save_state()?;
        Ok(())
    }
    fn move_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()> {
        self.copy_items(sources, destination)?;
        for source in sources {
            self.delete_item(source)?;
        }
        Ok(())
    }
    fn get_home_directory(&self) -> FileSystemResult<String> {
        let state = self.state.read()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку"))?;
        Ok(state.home_directory.clone())
    }
    fn get_system_folders(&self) -> FileSystemResult<Vec<FileSystemEntry>> {
        let home = self.get_home_directory()?;
        self.read_directory(&home)
    }
    fn read_file_content(&self, path: &str, max_size: Option<u64>) -> FileSystemResult<String> {
        let node = self.find_node(path)?;
        match node {
            VfsNode::File { content, .. } => {
                if let Some(max) = max_size {
                    if content.len() as u64 > max {
                        return Err(FileSystemError::new(format!("Файл слишком большой (>{} байт)", max)));
                    }
                }
                match String::from_utf8(content.clone()) {
                    Ok(text) => Ok(text),
                    Err(_) => {
                        Ok(base64::encode(&content))
                    }
                }
            }
            VfsNode::Directory { .. } => {
                Err(FileSystemError::new(format!("'{}' является директорией", path)))
            }
        }
    }
    fn open_file(&self, _path: &str) -> FileSystemResult<()> {
        Ok(())
    }
    fn reveal_in_finder(&self, _path: &str) -> FileSystemResult<()> {
        Ok(())
    }
    fn normalize_path(&self, path: &str) -> FileSystemResult<String> {
        let normalized = self.normalize_path_internal(path);
        self.find_node(&normalized)?;
        Ok(normalized)
    }
    fn get_path_suggestions(&self, partial_path: &str) -> FileSystemResult<Vec<String>> {
        let normalized = self.normalize_path_internal(partial_path);
        if partial_path.ends_with('/') {
            let entries = self.read_directory(&normalized)?;
            return Ok(entries.iter()
                .filter(|e| e.is_dir)
                .map(|e| e.path.clone())
                .collect());
        }
        let (parent_parts, prefix) = self.find_parent_and_name(partial_path)?;
        let parent_path = if parent_parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parent_parts.join("/"))
        };
        let entries = self.read_directory(&parent_path)?;
        Ok(entries.iter()
            .filter(|e| e.is_dir && e.name.starts_with(&prefix))
            .map(|e| e.path.clone())
            .collect())
    }
    fn open_terminal(&self, _path: &str) -> FileSystemResult<()> {
        Ok(())
    }
}
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
mod base64 {
    pub fn encode(data: &[u8]) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        for chunk in data.chunks(3) {
            let b1 = chunk[0];
            let b2 = chunk.get(1).copied().unwrap_or(0);
            let b3 = chunk.get(2).copied().unwrap_or(0);
            let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
            let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let c1 = chars.chars().nth(((n >> 18) & 63) as usize).unwrap();
            let c2 = chars.chars().nth(((n >> 12) & 63) as usize).unwrap();
            let c3 = if chunk.len() > 1 { chars.chars().nth(((n >> 6) & 63) as usize).unwrap() } else { '=' };
            let c4 = if chunk.len() > 2 { chars.chars().nth((n & 63) as usize).unwrap() } else { '=' };
            write!(&mut result, "{}{}{}{}", c1, c2, c3, c4).unwrap();
        }
        result
    }
}
```


---


## 📄 `src-tauri/src/application/filesystem_api.rs`

**Tokens:** ~0 | **Lines:** 0
```rust

```


---


## 📄 `src-tauri/src/application/mod.rs`

**Tokens:** ~3 | **Lines:** 1
```rust
mod filesystem_api;
```


---


## 📄 `src-tauri/src/commands.rs`

**Tokens:** ~1449 | **Lines:** 215
```rust
use crate::api::virtual_fs::VirtualFileSystem;
use crate::api::RealFileSystem;
use crate::config::{AppConfig, Bookmark, FileSystemBackend, UIState};
use crate::core::{FileSystem, FileSystemEntry};
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
static APP_CONFIG: Lazy<Arc<RwLock<AppConfig>>> = Lazy::new(|| {
    let config = AppConfig::load().unwrap_or_default();
    Arc::new(RwLock::new(config))
});
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
#[tauri::command]
pub fn get_config() -> Result<AppConfig, String> {
    let config = APP_CONFIG.read().unwrap();
    Ok(config.clone())
}
#[tauri::command]
pub fn update_config(new_config: AppConfig) -> Result<(), String> {
    new_config.save()?;
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
#[tauri::command]
pub fn get_bookmarks() -> Result<Vec<Bookmark>, String> {
    let config = APP_CONFIG.read().unwrap();
    Ok(config.bookmarks.clone())
}
#[tauri::command]
pub fn add_bookmark(path: String, name: Option<String>) -> Result<Bookmark, String> {
    let mut config = APP_CONFIG.write().unwrap();
    if config.bookmarks.iter().any(|b| b.path == path) {
        return Err("Bookmark with this path already exists".to_string());
    }
    let bookmark_name = name.unwrap_or_else(|| {
        std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Bookmark")
            .to_string()
    });
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let id = format!("bookmark_{}", timestamp);
    let bookmark = Bookmark {
        id: id.clone(),
        name: bookmark_name,
        path: path.clone(),
        created_at: timestamp,
    };
    config.bookmarks.push(bookmark.clone());
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
#[tauri::command]
pub fn get_ui_state() -> Result<UIState, String> {
    let config = APP_CONFIG.read().unwrap();
    let ui_state = config.ui_state.clone();
    Ok(ui_state)
}
#[tauri::command]
pub fn save_ui_state(ui_state: UIState) -> Result<(), String> {
    let mut config = APP_CONFIG.write().unwrap();
    config.ui_state = ui_state;
    let config_path = crate::config::AppConfig::config_path()?;
    config.save()?;
    Ok(())
}
```


---


## 📄 `src-tauri/src/config.rs`

**Tokens:** ~1021 | **Lines:** 177
```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileSystemBackend {
    Real,
    Virtual,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bookmark {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TabState {
    pub id: u64,
    pub path: Vec<String>,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub maximized: bool,
}
impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            x: None,
            y: None,
            maximized: false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarState {
    #[serde(default)]
    pub expanded_folders: Vec<String>,
    #[serde(default = "default_true")]
    pub quick_access_expanded: bool,
    #[serde(default = "default_true")]
    pub folder_tree_expanded: bool,
    #[serde(default)]
    pub favorites_expanded: bool,
}
fn default_true() -> bool {
    true
}
impl Default for SidebarState {
    fn default() -> Self {
        Self {
            expanded_folders: vec![],
            quick_access_expanded: true,
            folder_tree_expanded: true,
            favorites_expanded: false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIState {
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
    #[serde(default = "default_preview_width")]
    pub preview_width: u32,
    #[serde(default)]
    pub tabs: Vec<TabState>,
    #[serde(default)]
    pub active_tab_id: Option<u64>,
    #[serde(default)]
    pub last_path: Option<Vec<String>>,
    #[serde(default)]
    pub window: WindowState,
    #[serde(default)]
    pub sidebar: SidebarState,
}
fn default_sidebar_width() -> u32 {
    240
}
fn default_preview_width() -> u32 {
    300
}
impl Default for UIState {
    fn default() -> Self {
        Self {
            sidebar_width: default_sidebar_width(),
            preview_width: default_preview_width(),
            tabs: vec![],
            active_tab_id: None,
            last_path: None,
            window: WindowState::default(),
            sidebar: SidebarState::default(),
        }
    }
}
impl Default for FileSystemBackend {
    fn default() -> Self {
        FileSystemBackend::Real
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub filesystem_backend: FileSystemBackend,
    #[serde(default = "default_show_hidden")]
    pub show_hidden_files: bool,
    #[serde(default = "default_view_mode")]
    pub default_view_mode: String,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub bookmarks: Vec<Bookmark>,
    #[serde(default)]
    pub ui_state: UIState,
}
fn default_show_hidden() -> bool {
    false
}
fn default_view_mode() -> String {
    "grid".to_string()
}
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            filesystem_backend: FileSystemBackend::default(),
            show_hidden_files: false,
            default_view_mode: "grid".to_string(),
            theme: "luna".to_string(),
            bookmarks: Vec::new(),
            ui_state: UIState::default(),
        }
    }
}
impl AppConfig {
    pub fn config_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| "Could not determine config directory".to_string())?;
        let app_config_dir = config_dir.join("vfdir");
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        Ok(app_config_dir.join("config.json"))
    }
    pub fn load() -> Result<Self, String> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        Ok(config)
    }
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }
}
```


---


## 📄 `src-tauri/src/core/directory.rs`

**Tokens:** ~231 | **Lines:** 38
```rust
use crate::core::metadata::Metadata;
use crate::core::node::Node;
pub struct  Directory{
    name: String,
    child: Vec<Node>,
    metadata: Metadata
}
impl Directory {
    pub fn new(name: String) -> Self {
        Self { name, child: Vec::new(), metadata: Metadata::new(0, 0, 0) }
    }
    pub fn find_child(&self, n: &str) -> Option<&String>{
        if n == self.name { Some(&self.name) }
        else { None }
    }
    pub fn find_name_mut(&mut self, n: &str) -> Option<&mut String>{
        if n == self.name { Some(&mut self.name) }
        else { None }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn child(&self) -> &Vec<Node> {
        &self.child
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_child(&mut self, child: Vec<Node>) {
        self.child = child;
    }
    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }
}
```


---


## 📄 `src-tauri/src/core/file.rs`

**Tokens:** ~218 | **Lines:** 38
```rust
use crate::core::metadata::Metadata;
pub struct File {
    name: String,
    content: Vec<u8>,
    metadata: Metadata
}
impl File {
    pub fn new(name: String) -> Self{
        File{name, content: vec![0], metadata: Metadata::new(0, 0, 0)}
    }
    pub fn with_content(name: String, content: Vec<u8>) -> Self{
        File{name, content, metadata: Metadata::new(0, 0, 0)}
    }
    pub fn read(&self) -> &[u8] {
        &self.content
    }
    pub fn write(&mut self, data: &[u8]) {
        self.content = Vec::from(data);
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_content(&mut self, content: Vec<u8>) {
        self.content = content;
    }
    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }
}
```


---


## 📄 `src-tauri/src/core/filesystem.rs`

**Tokens:** ~465 | **Lines:** 58
```rust
use serde::{Deserialize, Serialize};
use std::fmt;
pub type FileSystemResult<T> = Result<T, FileSystemError>;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
pub trait FileSystem: Send + Sync {
    fn read_directory(&self, path: &str) -> FileSystemResult<Vec<FileSystemEntry>>;
    fn get_file_info(&self, path: &str) -> FileSystemResult<FileSystemEntry>;
    fn delete_item(&self, path: &str) -> FileSystemResult<()>;
    fn rename_item(&self, old_path: &str, new_name: &str) -> FileSystemResult<()>;
    fn create_folder(&self, path: &str, name: &str) -> FileSystemResult<()>;
    fn copy_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()>;
    fn move_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()>;
    fn get_home_directory(&self) -> FileSystemResult<String>;
    fn get_system_folders(&self) -> FileSystemResult<Vec<FileSystemEntry>>;
    fn read_file_content(&self, path: &str, max_size: Option<u64>) -> FileSystemResult<String>;
    fn open_file(&self, path: &str) -> FileSystemResult<()>;
    fn reveal_in_finder(&self, path: &str) -> FileSystemResult<()>;
    fn normalize_path(&self, path: &str) -> FileSystemResult<String>;
    fn get_path_suggestions(&self, partial_path: &str) -> FileSystemResult<Vec<String>>;
    fn open_terminal(&self, path: &str) -> FileSystemResult<()>;
}
```


---


## 📄 `src-tauri/src/core/metadata.rs`

**Tokens:** ~59 | **Lines:** 12
```rust
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Clone)]
pub struct Metadata {
    created_at: u64,
    modified_at: u64,
    size: usize
}
impl Metadata{
    pub fn new(created_at: u64, modified_at: u64, size: usize) -> Metadata{
        Self{created_at, modified_at, size}
    }
}
```


---


## 📄 `src-tauri/src/core/mod.rs`

**Tokens:** ~35 | **Lines:** 7
```rust
mod metadata;
mod file;
mod directory;
mod node;
pub mod filesystem;
pub mod search;
pub use filesystem::{FileSystem, FileSystemEntry, FileSystemError, FileSystemResult};
```


---


## 📄 `src-tauri/src/core/node.rs`

**Tokens:** ~288 | **Lines:** 54
```rust
use crate::core::directory::Directory;
use crate::core::file::File;
use crate::core::metadata::Metadata;
pub enum  Node{
    File(File),
    Directory(Directory)
}
impl Node {
    pub fn name(&self) -> &str{
        match self { Node::Directory(d) => d.name(), Node::File(f) => f.name() }
    }
    pub fn is_dir(&self) -> bool {
        match self {
            Node::Directory(d) => true,
            _ => false
        }
    }
    pub fn is_file(&self) -> bool{
        match self {
            Node::File(f) => true,
            _ => false
        }
    }
    pub fn metadata(&self) -> &Metadata{
        match self {
            Node::File(f) => f.metadata(),
            Node::Directory(d) => d.metadata()
        }
    }
    pub fn as_file(&self) -> Option<&File> {
        match self {
            Node::File(f) => Some(f),
            _ => None
        }
    }
    pub fn as_dir(&self) -> Option<&Directory> {
        match self {
            Node::Directory(d) => Some(d),
            _ => None
        }
    }
    pub fn as_file_mut(&mut self) -> Option<&mut File>{
        match self {
            Node::File(f) => Some(f),
            _ => None
        }
    }
    pub fn as_dir_mut(&mut self) -> Option<&mut Directory>{
        match self {
            Node::Directory(d) => Some(d),
            _ => None
        }
    }
}
```


---


## 📄 `src-tauri/src/core/search/and_specification.rs`

**Tokens:** ~99 | **Lines:** 16
```rust
use crate::core::FileSystemEntry;
use crate::core::search::trait_file_specification::FileSpecification;
pub struct AndSpecification {
    specs: Vec<Box<dyn FileSpecification>>,
}
impl AndSpecification {
    pub fn new(specs: Vec<Box<dyn FileSpecification>>) -> Self {
        Self { specs }
    }
}
impl FileSpecification for AndSpecification {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
        let i = self.specs.iter().all(|s| s.is_satisfied_by(item));
        i
    }
}
```


---


## 📄 `src-tauri/src/core/search/enums.rs`

**Tokens:** ~387 | **Lines:** 62
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TextMatchMode {
    Exact,
    Contains,
    Regex,
    Fuzzy(usize),
}
use regex::Regex;
use crate::core::FileSystemEntry;
use crate::core::search::trait_file_specification::FileSpecification;
use crate::core::search::size_spec::SizeSpec;
use crate::core::search::extension_spec::ExtensionSpecification;
use crate::core::search::and_specification::AndSpecification;
use strsim::levenshtein;
pub struct NameSpecification {
    pattern: String,
    mode: TextMatchMode,
    compiled_regex: Option<Regex>,
}
impl NameSpecification {
    pub fn new(pattern: String, mode: TextMatchMode) -> Result<Self, String> {
        let compiled_regex = match mode {
            TextMatchMode::Regex => {
                match Regex::new(&pattern) {
                    Ok(regex) => Some(regex),
                    Err(e) => return Err(format!("invalid regex {}", e)),
                }
            }
            _ => None,
        };
        Ok(Self {
            pattern,
            mode,
            compiled_regex,
        })
    }
}
impl FileSpecification for NameSpecification {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool  {
        match self.mode {
            TextMatchMode::Regex => {
                if let Some(regex) = &self.compiled_regex { regex.is_match(&item.name) } else { panic!("Regex not compiled!"); }
            }
            TextMatchMode::Exact => {
                item.name.to_lowercase().eq(&self.pattern)
            }
            TextMatchMode::Fuzzy(max_distance) => {
                let pattern_lc = self.pattern.to_lowercase();
                let name_lc = item.name.to_lowercase();
                let d = levenshtein(&pattern_lc, &name_lc);
                d <= max_distance
            }
            TextMatchMode::Contains => {
                item.name.to_lowercase().contains(&self.pattern.to_lowercase())
            }
        }
    }
}
pub struct SearchQuery {
    pub root_spec: Box<dyn FileSpecification>,
    pub recursive: bool,
}
```


---


## 📄 `src-tauri/src/core/search/extension_spec.rs`

**Tokens:** ~117 | **Lines:** 16
```rust
use crate::core::FileSystemEntry;
use crate::core::search::trait_file_specification::FileSpecification;
pub struct ExtensionSpecification {
    pub extension: String,
}
impl ExtensionSpecification {
    pub fn new(extension: String) -> Self {
        let ext = if extension.starts_with('.') { extension.to_lowercase() } else { format!(".{}", extension.to_lowercase()) };
        Self {extension: ext}
    }
}
impl FileSpecification for ExtensionSpecification {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
        item.name.to_lowercase().ends_with(&self.extension)
    }
}
```


---


## 📄 `src-tauri/src/core/search/mod.rs`

**Tokens:** ~40 | **Lines:** 8
```rust
pub mod enums;
pub mod trait_file_specification;
pub mod and_specification;
pub mod name_contains_spec;
pub mod extension_spec;
pub mod size_spec;
pub mod search_service;
pub mod sq_n_sqb;
```


---


## 📄 `src-tauri/src/core/search/name_contains_spec.rs`

**Tokens:** ~59 | **Lines:** 8
```rust
use crate::core::FileSystemEntry;
use crate::core::search::trait_file_specification::FileSpecification;
pub struct NameContainsSpec(pub String);
impl FileSpecification for NameContainsSpec {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
        item.name.to_lowercase().contains(&self.0.to_lowercase())
    }
}
```


---


## 📄 `src-tauri/src/core/search/search_service.rs`

**Tokens:** ~216 | **Lines:** 33
```rust
use crate::core::{FileSystem, FileSystemEntry};
use crate::core::search::enums::SearchQuery;
pub struct SearchService<FS: FileSystem> {
    file_system: FS,
}
impl<FS: FileSystem> SearchService<FS> {
    pub fn new(file_system: FS) -> Self {
        Self { file_system }
    }
    pub fn search(&self, root_path: &str, query: SearchQuery) -> Result<Vec<FileSystemEntry>, String> {
        let mut new_v = Vec::new();
        self.search_recursive(root_path, &query, &mut new_v)?;
        Ok(new_v)
    }
    fn search_recursive(
        &self,
        path: &str,
        query: &SearchQuery,
        results: &mut Vec<FileSystemEntry>,
    ) -> Result<(), String> {
        let entries = self.file_system.read_directory(path).unwrap();
        for entry in entries {
            if entry.is_file {
                if query.root_spec.is_satisfied_by(&entry) {
                    results.push(entry);
                } else if entry.is_dir && query.recursive{
                    self.search_recursive(&entry.path, query, results)?;
                }
            }
        }
        Ok(())
    }
}
```


---


## 📄 `src-tauri/src/core/search/size_spec.rs`

**Tokens:** ~212 | **Lines:** 33
```rust
use crate::core::FileSystemEntry;
use crate::core::search::trait_file_specification::FileSpecification;
pub struct SizeSpec {
    pub min_bytes: Option<u64>,
    pub max_bytes: Option<u64>,
}
impl SizeSpec {
    pub fn new(min_bytes: Option<u64>, max_bytes: Option<u64>) -> Self {
        Self { min_bytes, max_bytes }
    }
}
impl FileSpecification for SizeSpec {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
        match item.size {
            None => false,
            Some(m) => {
                if self.min_bytes.is_none() && self.max_bytes.is_none() {
                    true
                } else if let Some(min) = self.min_bytes {
                    if let Some(max) = self.max_bytes {
                        m >= min && m <= max
                    } else {
                        m >= min
                    }
                } else if let Some(max) = self.max_bytes {
                    m <= max
                } else {
                    false
                }
            }
        }
    }
}
```


---


## 📄 `src-tauri/src/core/search/sq_n_sqb.rs`

**Tokens:** ~316 | **Lines:** 48
```rust
use crate::core::search::and_specification::AndSpecification;
use crate::core::search::enums::{NameSpecification, SearchQuery, TextMatchMode};
use crate::core::search::extension_spec::ExtensionSpecification;
use crate::core::search::size_spec::SizeSpec;
use crate::core::search::trait_file_specification::FileSpecification;
pub struct SearchQueryBuilder {
    specs: Vec<Box<dyn FileSpecification>>,
    recursive: bool,
}
impl SearchQueryBuilder {
    pub fn new() -> Self {
        Self {
            specs: Vec::new(),
            recursive: false,
        }
    }
    pub fn with_name(mut self, pattern: &str, mode: TextMatchMode) -> Self {
        match NameSpecification::new(pattern.to_string(), mode) {
            Ok(s) => self.specs.push(Box::new(s)),
            Err(e) => panic!("Warning: Invalid pattern: {}", e)
        }
        self
    }
    pub fn with_size_range(mut self, min_bytes: Option<u64>, max_bytes: Option<u64>) -> Self {
        self.specs.push(Box::new(SizeSpec::new(min_bytes, max_bytes)));
        self
    }
    pub fn with_extension(mut self, extension: &str) -> Self {
        self.specs.push(Box::new(ExtensionSpecification::new(extension.to_string())));
        self
    }
    pub fn recursive(mut self, enable: bool) -> Self {
        self.recursive = enable;
        self
    }
    pub fn build(self) -> SearchQuery {
        let r = Box::new(AndSpecification::new(self.specs));
        SearchQuery {
            root_spec: r,
            recursive: self.recursive
        }
    }
}
impl Default for SearchQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```


---


## 📄 `src-tauri/src/core/search/trait_file_specification.rs`

**Tokens:** ~29 | **Lines:** 4
```rust
use crate::core::FileSystemEntry;
pub trait FileSpecification: Send + Sync {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool;
}
```


---


## 📄 `src-tauri/src/error.rs`

**Tokens:** ~48 | **Lines:** 10
```rust
#[derive(Debug, PartialEq)]
pub enum FsError {
    NotFound(String),
    AlreadyExists(String),
    InvalidOperation(String)
}
pub type FsResult<T> = Result<T, FsError>;
pub fn bug(errors: Vec<String>) -> FsError{
    unimplemented!()
}
```


---


## 📄 `src-tauri/src/lib.rs`

**Tokens:** ~213 | **Lines:** 45
```rust
pub mod api;
pub mod application;
pub mod core;
pub mod config;
mod error;
mod commands;
use commands::*;
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            read_directory,
            delete_item,
            rename_item,
            create_folder,
            copy_items,
            move_items,
            get_home_directory,
            get_file_info,
            open_file,
            reveal_in_finder,
            get_system_folders,
            read_file_content,
            normalize_path,
            get_path_suggestions,
            open_terminal,
            get_config,
            update_config,
            set_filesystem_backend,
            get_bookmarks,
            add_bookmark,
            remove_bookmark,
            rename_bookmark,
            get_ui_state,
            save_ui_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```


---


## 📄 `src-tauri/src/main.rs`

**Tokens:** ~43 | **Lines:** 10
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use llm_utl::api::Scan;
fn main() {
run();
}
fn run() {
    Scan::dir("../")
        .allow_only(vec!("**/*.rs"))
        .run().unwrap();
}
```


---


## 📄 `src-tauri/tests/contains_tests.rs`

**Tokens:** ~0 | **Lines:** 0
```rust

```


---


## 📄 `src-tauri/tests/extensionspec_tests.rs`

**Tokens:** ~10 | **Lines:** 1
```rust
use vfdir_lib::core::search::extension_spec::ExtensionSpecification;
```


---


## 📄 `src-tauri/tests/fuzzy_tests.rs`

**Tokens:** ~0 | **Lines:** 0
```rust

```


---


## 📄 `src-tauri/tests/mock_fs_tests.rs`

**Tokens:** ~0 | **Lines:** 0
```rust

```


---


## 📄 `src-tauri/tests/regex_tests.rs`

**Tokens:** ~0 | **Lines:** 0
```rust

```


---


## 📄 `src-tauri/tests/sqb_tests.rs`

**Tokens:** ~0 | **Lines:** 0
```rust

```


---


## 📄 `src-tauri/tests/tests.rs`

**Tokens:** ~0 | **Lines:** 0
```rust

```


---


## 📄 `src-tauri/tests/tests_w_size_lims.rs`

**Tokens:** ~8 | **Lines:** 1
```rust
use vfdir_lib::core::search::size_spec::SizeSpec;
```


---


## 📄 `src-tauri/tests/tests_w_specs.rs`

**Tokens:** ~136 | **Lines:** 20
```rust
mod tests {
    use vfdir_lib::core::FileSystemEntry;
    use vfdir_lib::core::search::and_specification::AndSpecification;
    use vfdir_lib::core::search::extension_spec::ExtensionSpecification;
    use vfdir_lib::core::search::name_contains_spec::NameContainsSpec;
    use vfdir_lib::core::search::size_spec::SizeSpec;
    use vfdir_lib::core::search::trait_file_specification::FileSpecification;
    fn create_test_file(name: &str) -> FileSystemEntry {
        FileSystemEntry {
            path: format!("/test/{}", name),
            name: name.to_string(),
            is_dir: false,
            is_file: true,
            size: Some(1024),
            modified: Some(1234567890),
            created: Some(1234567890),
            accessed: Some(1234567890),
        }
    }
}
```


---



<!-- End of chunk 1/1 -->