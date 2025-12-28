use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
// Импортируем типы из вашего модуля
use crate::core::{FileSystem, FileSystemEntry, FileSystemError, FileSystemResult};

/// Узел виртуальной файловой системы
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
    /// Создать новый файл
    fn new_file(content: Vec<u8>) -> Self {
        let now = current_timestamp();
        VfsNode::File {
            content,
            created: now,
            modified: now,
        }
    }

    /// Создать новую директорию
    fn new_directory() -> Self {
        let now = current_timestamp();
        VfsNode::Directory {
            children: HashMap::new(),
            created: now,
            modified: now,
        }
    }

    /// Является ли узел директорией
    fn is_dir(&self) -> bool {
        matches!(self, VfsNode::Directory { .. })
    }

    /// Получить размер узла
    fn size(&self) -> u64 {
        match self {
            VfsNode::File { content, .. } => content.len() as u64,
            VfsNode::Directory { .. } => 0,
        }
    }

    /// Получить время создания
    fn created(&self) -> u64 {
        match self {
            VfsNode::File { created, .. } => *created,
            VfsNode::Directory { created, .. } => *created,
        }
    }

    /// Получить время модификации
    fn modified(&self) -> u64 {
        match self {
            VfsNode::File { modified, .. } => *modified,
            VfsNode::Directory { modified, .. } => *modified,
        }
    }
}

/// Состояние виртуальной файловой системы
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VfsState {
    root: VfsNode,
    home_directory: String,
}

impl Default for VfsState {
    fn default() -> Self {
        let mut root = VfsNode::new_directory();

        // Создаем базовую структуру директорий
        if let VfsNode::Directory { children, .. } = &mut root {
            let mut home = VfsNode::new_directory();

            if let VfsNode::Directory { children: home_children, .. } = &mut home {
                // Создаем стандартные папки
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

/// Виртуальная файловая система в памяти с персистентностью
pub struct VirtualFileSystem {
    state: Arc<RwLock<VfsState>>,
    persistence_path: PathBuf,
}

impl VirtualFileSystem {
    /// Создать новую виртуальную файловую систему
    ///
    /// # Аргументы
    /// * `persistence_path` - путь к файлу для сохранения состояния
    pub fn new(persistence_path: impl AsRef<Path>) -> FileSystemResult<Self> {
        let persistence_path = persistence_path.as_ref().to_path_buf();

        let state = if persistence_path.exists() {
            // Пытаемся загрузить состояние из файла
            Self::load_state(&persistence_path).unwrap_or_else(|e| {
                // Если файл поврежден, создаем новое состояние
                eprintln!("Предупреждение: не удалось загрузить состояние ({}), создается новое", e.message);
                VfsState::default()
            })
        } else {
            // Создаем новое состояние по умолчанию
            VfsState::default()
        };

        let vfs = Self {
            state: Arc::new(RwLock::new(state)),
            persistence_path,
        };

        // Сохраняем начальное состояние, если файла не было
        // Это создаст файл автоматически
        vfs.save_state()?;

        Ok(vfs)
    }

    /// Загрузить состояние из файла
    fn load_state(path: &Path) -> FileSystemResult<VfsState> {
        let data = std::fs::read(path)
            .map_err(|e| FileSystemError::new(format!("Не удалось прочитать файл состояния: {}", e)))?;

        // Проверяем, что файл не пустой
        if data.is_empty() {
            return Err(FileSystemError::new("Файл состояния пустой"));
        }

        let state: VfsState = serde_json::from_slice(&data)
            .map_err(|e| FileSystemError::new(format!("Не удалось десериализовать состояние: {}", e)))?;

        Ok(state)
    }

    /// Сохранить текущее состояние в файл
    fn save_state(&self) -> FileSystemResult<()> {
        let state = self.state.read()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку для чтения"))?;

        let data = serde_json::to_vec_pretty(&*state)
            .map_err(|e| FileSystemError::new(format!("Не удалось сериализовать состояние: {}", e)))?;

        std::fs::write(&self.persistence_path, data)
            .map_err(|e| FileSystemError::new(format!("Не удалось записать файл состояния: {}", e)))?;

        Ok(())
    }

    /// Нормализовать путь (убрать лишние слеши, обработать ~)
    fn normalize_path_internal(&self, path: &str) -> String {
        let path = if path.starts_with('~') {
            let state = self.state.read().unwrap();
            path.replacen('~', &state.home_directory, 1)
        } else {
            path.to_string()
        };

        // Убираем двойные слеши и приводим к стандартному виду
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parts.join("/"))
        }
    }

    /// Найти узел по пути
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

    /// Найти родительский узел и имя элемента
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

    /// Мутабельный доступ к узлу по пути
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
        // Копируем узлы
        let nodes_to_copy: Vec<(String, VfsNode)> = sources.iter()
            .map(|src| {
                let name = src.split('/').filter(|s| !s.is_empty()).last()
                    .ok_or_else(|| FileSystemError::new("Некорректный путь источника"))?
                    .to_string();
                let node = self.find_node(src)?;
                Ok((name, node))
            })
            .collect::<FileSystemResult<Vec<_>>>()?;

        // Вставляем в место назначения
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

    fn copy_with_custom_name(
        &self,
        source: &str,
        destination_dir: &str,
        new_name: &str,
    ) -> FileSystemResult<()> {
        // Получаем узел источника
        let node = self.find_node(source)?;

        // Вставляем в место назначения с новым именем
        let normalized_dest = self.normalize_path_internal(destination_dir);
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
                children.insert(new_name.to_string(), node);
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
        // Сначала копируем
        self.copy_items(sources, destination)?;

        // Затем удаляем оригиналы
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

                // Пытаемся интерпретировать как UTF-8 текст
                match String::from_utf8(content.clone()) {
                    Ok(text) => Ok(text),
                    Err(_) => {
                        // Если не UTF-8, возвращаем base64
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
        // В виртуальной файловой системе это заглушка
        Ok(())
    }

    fn reveal_in_finder(&self, _path: &str) -> FileSystemResult<()> {
        // В виртуальной файловой системе это заглушка
        Ok(())
    }

    fn normalize_path(&self, path: &str) -> FileSystemResult<String> {
        let normalized = self.normalize_path_internal(path);

        // Проверяем существование
        self.find_node(&normalized)?;

        Ok(normalized)
    }

    fn get_path_suggestions(&self, partial_path: &str) -> FileSystemResult<Vec<String>> {
        let normalized = self.normalize_path_internal(partial_path);

        // Если путь заканчивается на /, показываем содержимое этой директории
        if partial_path.ends_with('/') {
            let entries = self.read_directory(&normalized)?;
            return Ok(entries.iter()
                .filter(|e| e.is_dir)
                .map(|e| e.path.clone())
                .collect());
        }

        // Иначе ищем родительскую директорию и фильтруем
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
        // В виртуальной файловой системе это заглушка
        Ok(())
    }
}

/// Получить текущий timestamp в миллисекундах
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// Для использования base64
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_read_directory() {
        let vfs = VirtualFileSystem::new("test_vfs_state.json").unwrap();

        let home = vfs.get_home_directory().unwrap();
        vfs.create_folder(&home, "test_folder").unwrap();

        let entries = vfs.read_directory(&home).unwrap();
        assert!(entries.iter().any(|e| e.name == "test_folder" && e.is_dir));
    }

    #[test]
    fn test_delete_folder() {
        let vfs = VirtualFileSystem::new("test_vfs_state2.json").unwrap();

        let home = vfs.get_home_directory().unwrap();
        vfs.create_folder(&home, "to_delete").unwrap();

        let path = format!("{}/to_delete", home);
        vfs.delete_item(&path).unwrap();

        let entries = vfs.read_directory(&home).unwrap();
        assert!(!entries.iter().any(|e| e.name == "to_delete"));
    }

    #[test]
    fn test_rename() {
        let vfs = VirtualFileSystem::new("test_vfs_state3.json").unwrap();

        let home = vfs.get_home_directory().unwrap();
        vfs.create_folder(&home, "old_name").unwrap();

        let old_path = format!("{}/old_name", home);
        vfs.rename_item(&old_path, "new_name").unwrap();

        let entries = vfs.read_directory(&home).unwrap();
        assert!(entries.iter().any(|e| e.name == "new_name"));
        assert!(!entries.iter().any(|e| e.name == "old_name"));
    }
}