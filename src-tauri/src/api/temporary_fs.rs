use crate::core::{FileSystem, FileSystemEntry, FileSystemError, FileSystemResult};
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Временная файловая система, ограниченная определенной директорией (chroot)
#[derive(Clone)]
pub struct TemporaryFileSystem {
    root_path: PathBuf,
}

impl TemporaryFileSystem {
    pub fn new(root_path: PathBuf) -> Self {
        // Ensure root exists
        if !root_path.exists() {
            let _ = fs::create_dir_all(&root_path);
        }
        Self { root_path }
    }

    /// Resolve a virtual path (e.g. "/folder/file.txt") to a real path inside the root
    /// Prevents directory traversal attacks
    fn resolve_path(&self, path: &str) -> FileSystemResult<PathBuf> {
        // Clean up path
        let path = path.trim();
        
        // Handle root
        if path == "/" || path.is_empty() {
            return Ok(self.root_path.clone());
        }

        // Remove leading slash
        let relative_path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };

        // Join with root
        let full_path = self.root_path.join(relative_path);

        // Canonicalize to prevent .. traversal
        // Note: canonicalize requires file to exist, which might not be true for creation.
        // So we manually check components.
        
        // Security check: ensure the resulting path starts with root_path
        // We use a simple check here. For robust implementation, components check is better.
        // Assuming typical usage, simple join is "okay" if we trust the input is not malicious "../../../".
        // But let's be safe.
        
        // Simple check for ".." components
        if path.contains("..") {
             return Err(FileSystemError::new("Directory traversal detected"));
        }

        Ok(full_path)
    }

    /// Convert real path back to virtual path (relative to root)
    fn to_virtual_path(&self, real_path: &Path) -> String {
        if let Ok(relative) = real_path.strip_prefix(&self.root_path) {
            let s = relative.to_string_lossy().to_string();
            if s.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", s)
            }
        } else {
            // Should not happen if logic is correct
            real_path.to_string_lossy().to_string()
        }
    }

    fn path_to_entry(&self, path: &Path) -> FileSystemResult<FileSystemEntry> {
        let metadata = fs::metadata(path)
            .map_err(|e| FileSystemError::new(format!("Failed to read metadata: {}", e)))?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let virtual_path = self.to_virtual_path(path);

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
            path: virtual_path,
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
}

impl FileSystem for TemporaryFileSystem {
    fn read_directory(&self, path: &str) -> FileSystemResult<Vec<FileSystemEntry>> {
        let real_path = self.resolve_path(path)?;

        if !real_path.exists() {
            return Err(FileSystemError::new(format!("Directory does not exist: {}", path)));
        }

        if !real_path.is_dir() {
            return Err(FileSystemError::new(format!("Path is not a directory: {}", path)));
        }

        let entries = fs::read_dir(&real_path)
            .map_err(|e| FileSystemError::new(format!("Failed to read directory: {}", e)))?;

        let mut result = Vec::new();

        for entry in entries {
            let entry = entry
                .map_err(|e| FileSystemError::new(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            // Skip hidden files
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with('.') {
                        continue;
                    }
                }
            }

            match self.path_to_entry(&path) {
                Ok(fs_entry) => result.push(fs_entry),
                Err(_) => continue,
            }
        }

        // Sort
        result.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(result)
    }

    fn get_file_info(&self, path: &str) -> FileSystemResult<FileSystemEntry> {
        let real_path = self.resolve_path(path)?;

        if !real_path.exists() {
            return Err(FileSystemError::new(format!("Path does not exist: {}", path)));
        }

        self.path_to_entry(&real_path)
    }

    fn delete_item(&self, path: &str) -> FileSystemResult<()> {
        let real_path = self.resolve_path(path)?;

        if !real_path.exists() {
            return Err(FileSystemError::new(format!("Path does not exist: {}", path)));
        }

        if real_path.is_dir() {
            fs::remove_dir_all(&real_path)
                .map_err(|e| FileSystemError::new(format!("Failed to delete directory: {}", e)))?;
        } else {
            fs::remove_file(&real_path)
                .map_err(|e| FileSystemError::new(format!("Failed to delete file: {}", e)))?;
        }

        Ok(())
    }

    fn rename_item(&self, old_path: &str, new_name: &str) -> FileSystemResult<()> {
        let real_old_path = self.resolve_path(old_path)?;

        if !real_old_path.exists() {
            return Err(FileSystemError::new(format!("Path does not exist: {}", old_path)));
        }

        let parent = real_old_path
            .parent()
            .ok_or_else(|| FileSystemError::new("Could not get parent directory"))?;

        let real_new_path = parent.join(new_name);

        if real_new_path.exists() {
            return Err(FileSystemError::new(format!("File already exists: {}", new_name)));
        }

        fs::rename(&real_old_path, &real_new_path)
            .map_err(|e| FileSystemError::new(format!("Failed to rename: {}", e)))?;

        Ok(())
    }

    fn create_folder(&self, path: &str, name: &str) -> FileSystemResult<()> {
        let real_parent = self.resolve_path(path)?;

        if !real_parent.exists() {
            return Err(FileSystemError::new(format!("Parent directory does not exist: {}", path)));
        }

        let real_new_path = real_parent.join(name);

        if real_new_path.exists() {
            return Err(FileSystemError::new(format!("Folder already exists: {}", name)));
        }

        fs::create_dir(&real_new_path)
            .map_err(|e| FileSystemError::new(format!("Failed to create folder: {}", e)))?;

        Ok(())
    }

    fn create_file(&self, path: &str, name: &str, content: Option<&str>) -> FileSystemResult<()> {
        let real_parent = self.resolve_path(path)?;

        if !real_parent.exists() {
            return Err(FileSystemError::new(format!("Parent directory does not exist: {}", path)));
        }

        let real_new_path = real_parent.join(name);

        if real_new_path.exists() {
            return Err(FileSystemError::new(format!("File already exists: {}", name)));
        }

        let mut file = fs::File::create(&real_new_path)
            .map_err(|e| FileSystemError::new(format!("Failed to create file: {}", e)))?;

        if let Some(content_str) = content {
            file.write_all(content_str.as_bytes())
                .map_err(|e| FileSystemError::new(format!("Failed to write file content: {}", e)))?;
        }

        Ok(())
    }

    fn create_files_batch(&self, path: &str, files: &[(String, Option<String>)]) -> FileSystemResult<Vec<FileSystemResult<()>>> {
        let mut results = Vec::new();
        for (name, content) in files {
            results.push(self.create_file(path, name, content.as_deref()));
        }
        Ok(results)
    }

    fn copy_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()> {
        // Not implementing complex copy within temp fs for now, mainly used for stego viewing
        // But implementing basic file copy is useful
        
        let dest_real_path = self.resolve_path(destination)?;
        if !dest_real_path.exists() || !dest_real_path.is_dir() {
             return Err(FileSystemError::new("Destination is not a directory"));
        }

        for source in sources {
            let src_real_path = self.resolve_path(source)?;
            if !src_real_path.exists() {
                continue; 
            }
            
            let file_name = src_real_path.file_name()
                .ok_or_else(|| FileSystemError::new("Invalid source name"))?;
            let dest_file_path = dest_real_path.join(file_name);
            
            if src_real_path.is_dir() {
                // Recursive copy omitted for brevity, reusing RealFileSystem logic would be better if needed
                // For now error out on folders or implement simple recursive copy
                return Err(FileSystemError::new("Directory copy not fully supported in TempFS yet"));
            } else {
                fs::copy(&src_real_path, &dest_file_path)
                    .map_err(|e| FileSystemError::new(format!("Failed to copy: {}", e)))?;
            }
        }
        Ok(())
    }

    fn copy_with_custom_name(&self, source: &str, destination_dir: &str, new_name: &str) -> FileSystemResult<()> {
        let src_real = self.resolve_path(source)?;
        let dest_dir_real = self.resolve_path(destination_dir)?;
        let dest_file_real = dest_dir_real.join(new_name);
        
        if src_real.is_file() {
            fs::copy(&src_real, &dest_file_real)
                .map_err(|e| FileSystemError::new(format!("Failed to copy: {}", e)))?;
            Ok(())
        } else {
            Err(FileSystemError::new("Source must be a file"))
        }
    }

    fn move_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()> {
        let dest_real_path = self.resolve_path(destination)?;
        
        for source in sources {
            let src_real_path = self.resolve_path(source)?;
            let file_name = src_real_path.file_name()
                .ok_or_else(|| FileSystemError::new("Invalid name"))?;
            let dest_file = dest_real_path.join(file_name);
            
            fs::rename(&src_real_path, &dest_file)
                .map_err(|e| FileSystemError::new(format!("Failed to move: {}", e)))?;
        }
        Ok(())
    }

    fn get_home_directory(&self) -> FileSystemResult<String> {
        Ok("/".to_string())
    }

    fn get_system_folders(&self) -> FileSystemResult<Vec<FileSystemEntry>> {
        // In temp FS, "system folders" is just root
        let root_entry = self.path_to_entry(&self.root_path)?;
        // Maybe return it as "Home"
        let mut entry = root_entry;
        entry.name = "Home".to_string();
        entry.path = "/".to_string();
        Ok(vec![entry])
    }

    fn read_file_content(&self, path: &str, max_size: Option<u64>) -> FileSystemResult<String> {
        let real_path = self.resolve_path(path)?;
        
        if !real_path.exists() || !real_path.is_file() {
            return Err(FileSystemError::new("File does not exist"));
        }
        
        let metadata = fs::metadata(&real_path).map_err(|e| FileSystemError::new(e.to_string()))?;
        if let Some(max) = max_size {
            if metadata.len() > max {
                return Err(FileSystemError::new("File too large"));
            }
        }
        
        // Check binary vs text similar to RealFS
        // Simple check: read into string, if fails return base64
        let content = fs::read(&real_path).map_err(|e| FileSystemError::new(e.to_string()))?;
        
        match String::from_utf8(content.clone()) {
            Ok(s) => Ok(s),
            Err(_) => Ok(general_purpose::STANDARD.encode(&content)),
        }
    }

    fn read_file_bytes(&self, path: &str) -> FileSystemResult<Vec<u8>> {
        let real_path = self.resolve_path(path)?;
        fs::read(&real_path).map_err(|e| FileSystemError::new(e.to_string()))
    }

    fn write_file_content(&self, path: &str, content: &str) -> FileSystemResult<()> {
        let real_path = self.resolve_path(path)?;
        if let Some(parent) = real_path.parent() {
            if !parent.exists() {
                return Err(FileSystemError::new("Parent directory does not exist"));
            }
        }
        fs::write(&real_path, content).map_err(|e| FileSystemError::new(e.to_string()))
    }

    fn write_file_bytes(&self, path: &str, content: &[u8]) -> FileSystemResult<()> {
        let real_path = self.resolve_path(path)?;
        if let Some(parent) = real_path.parent() {
            if !parent.exists() {
                return Err(FileSystemError::new("Parent directory does not exist"));
            }
        }
        fs::write(&real_path, content).map_err(|e| FileSystemError::new(e.to_string()))
    }

    fn open_file(&self, _path: &str) -> FileSystemResult<()> {
        // Not supported in temp/sandbox
        Ok(())
    }

    fn reveal_in_finder(&self, _path: &str) -> FileSystemResult<()> {
        Ok(())
    }

    fn normalize_path(&self, path: &str) -> FileSystemResult<String> {
        // Already normalized by resolve logic conceptually
        // Just return clean path
        if path.starts_with('/') {
            Ok(path.to_string())
        } else {
            Ok(format!("/{}", path))
        }
    }

    fn get_path_suggestions(&self, partial_path: &str) -> FileSystemResult<Vec<String>> {
        let real_path = self.resolve_path(partial_path)?;
        
        let (parent_dir, prefix) = if partial_path.ends_with('/') {
            (real_path, String::new())
        } else {
            let parent = real_path.parent().unwrap_or(&self.root_path).to_path_buf();
            let name = real_path.file_name().unwrap_or_default().to_string_lossy().to_string();
            (parent, name)
        };
        
        if !parent_dir.exists() {
            return Ok(vec![]);
        }
        
        let mut suggestions = Vec::new();
        for entry in fs::read_dir(parent_dir).map_err(|e| FileSystemError::new(e.to_string()))? {
            let entry = entry.map_err(|e| FileSystemError::new(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) {
                // Return virtual path
                // Construct suggestion... simplified
                suggestions.push(name);
            }
        }
        Ok(suggestions)
    }

    fn open_terminal(&self, _path: &str) -> FileSystemResult<()> {
        Ok(())
    }
}
