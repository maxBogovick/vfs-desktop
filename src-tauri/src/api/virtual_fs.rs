use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use std::cell::RefCell;
use ::base64::Engine;
// Импортируем типы из вашего модуля
use crate::core::{FileSystem, FileSystemEntry, FileSystemError, FileSystemResult};
use crate::config::VaultPaths;
use crate::state::APP_CONFIG;

// Thread-local storage for active recovery session
thread_local! {
    static RECOVERY_SESSION: RefCell<Option<crate::api::recovery::RecoverySession>> = RefCell::new(None);
}

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
pub struct VfsState {
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
                /*home_children.insert("Documents".to_string(), VfsNode::new_directory());
                home_children.insert("Downloads".to_string(), VfsNode::new_directory());
                home_children.insert("Pictures".to_string(), VfsNode::new_directory());
                home_children.insert("Desktop".to_string(), VfsNode::new_directory());*/
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
#[derive(Clone)]
pub struct VirtualFileSystem {
    // Old field kept for backward compatibility during transition
    state: Arc<RwLock<VfsState>>,
    persistence_path: PathBuf,

    // New vault-based security fields
    vault_status: Arc<RwLock<crate::api::security::VfsStatus>>,
    config_path: PathBuf,  // vault.meta
    data_path: PathBuf,    // vault.bin
    vault_enabled: bool,   // Feature flag for gradual migration
}

impl VirtualFileSystem {
    /// Create new VirtualFileSystem with configurable paths from AppConfig
    pub fn new_with_config() -> FileSystemResult<Self> {
        let config = APP_CONFIG.read().unwrap();
        let vault_paths = config.get_vault_paths()
            .map_err(|e| FileSystemError::new(format!("Failed to get vault paths: {}", e)))?;

        Self::new_with_paths(vault_paths)
    }

    /// Internal constructor with explicit paths
    fn new_with_paths(paths: VaultPaths) -> FileSystemResult<Self> {
        let persistence_path = paths.fs_json;
        let config_path = paths.vault_meta;
        let data_path = paths.vault_bin;

        // Determine initial vault status
        let vault_status = if config_path.exists() && data_path.exists() {
            crate::api::security::VfsStatus::Locked
        } else {
            crate::api::security::VfsStatus::NotInitialized
        };

        // Old state loading for backward compatibility
        let state = if persistence_path.exists() {
            Self::load_state(&persistence_path).unwrap_or_else(|e| {
                eprintln!("Предупреждение: не удалось загрузить состояние ({}), создается новое", e.message);
                VfsState::default()
            })
        } else {
            VfsState::default()
        };

        let vfs = Self {
            state: Arc::new(RwLock::new(state)),
            persistence_path,
            vault_status: Arc::new(RwLock::new(vault_status)),
            config_path,
            data_path,
            vault_enabled: true,
        };

        vfs.save_state()?;
        Ok(vfs)
    }

    /// Создать новую виртуальную файловую систему
    ///
    /// # Аргументы
    /// * `persistence_path` - путь к файлу для сохранения состояния
    ///
    /// # Deprecated
    /// Use `new_with_config()` instead for configurable vault paths
    #[deprecated(note = "Use new_with_config() instead")]
    pub fn new(persistence_path: impl AsRef<Path>) -> FileSystemResult<Self> {
        let persistence_path = persistence_path.as_ref().to_path_buf();

        // Prepare vault paths
        let parent_dir = persistence_path.parent()
            .ok_or_else(|| FileSystemError::new("Invalid persistence path"))?;
        let config_path = parent_dir.join("vault.meta");
        let data_path = parent_dir.join("vault.bin");

        // Determine initial vault status
        let vault_status = if config_path.exists() && data_path.exists() {
            // Vault is initialized but locked
            crate::api::security::VfsStatus::Locked
        } else {
            // Vault not initialized yet
            crate::api::security::VfsStatus::NotInitialized
        };

        // Old state loading for backward compatibility
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
            vault_status: Arc::new(RwLock::new(vault_status)),
            config_path,
            data_path,
            vault_enabled: true,  // Enable vault by default
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

    // ==================== VAULT SECURITY METHODS ====================

    /// Get current vault status
    pub fn get_vault_status(&self) -> String {
        let guard = self.vault_status.read().unwrap();
        match *guard {
            crate::api::security::VfsStatus::NotInitialized => "UNINITIALIZED".to_string(),
            crate::api::security::VfsStatus::Locked => "LOCKED".to_string(),
            crate::api::security::VfsStatus::Unlocked { .. } => "UNLOCKED".to_string(),
        }
    }

    /// Initialize vault with password (first-time setup)
    pub fn initialize_vault(&self, password: &str) -> Result<(), crate::api::vault_error::VaultError> {
        use crate::api::security::*;
        use crate::api::vault_error::VaultError;

        let mut status_guard = self.vault_status.write()
            .map_err(|_| VaultError::CryptoError("Lock poisoned".into()))?;

        // Check if already initialized
        if !matches!(*status_guard, VfsStatus::NotInitialized) {
            return Err(VaultError::CryptoError("Vault already initialized".into()));
        }

        // Generate salt and derive key
        let salt = generate_salt();
        let session = derive_master_key(password, &salt)?;

        // Create verification hash
        let verification_hash = create_verification_hash(&session);

        // Create and save config
        let config = VaultConfig::new(
            ::base64::engine::general_purpose::STANDARD.encode(&salt),
            verification_hash,
        );
        save_vault_config(&self.config_path, &config)?;

        // Get default VFS state
        let vfs_state = VfsState::default();

        // Serialize with Bincode
        let serialized = bincode::serialize(&vfs_state)
            .map_err(|e| VaultError::Serialization(format!("Bincode serialization failed: {}", e)))?;

        // Encrypt
        let encrypted_blob = encrypt_blob(&serialized, &session)?;

        // Save encrypted data atomically
        atomic_write(&self.data_path, &encrypted_blob)?;

        // Update status to Unlocked
        *status_guard = VfsStatus::Unlocked {
            fs: vfs_state,
            session,
        };

        Ok(())
    }

    /// Unlock vault with password
    pub fn unlock_vault(&self, password: &str) -> Result<(), crate::api::vault_error::VaultError> {
        use crate::api::security::*;
        use crate::api::vault_error::VaultError;

        let mut status_guard = self.vault_status.write()
            .map_err(|_| VaultError::CryptoError("Lock poisoned".into()))?;

        // Check if locked
        if !matches!(*status_guard, VfsStatus::Locked) {
            return Err(VaultError::CryptoError("Vault is not locked".into()));
        }

        // Load config
        let config = load_vault_config(&self.config_path)?;

        // Decode salt
        let salt = ::base64::engine::general_purpose::STANDARD.decode(&config.kdf_salt)
            .map_err(|e| VaultError::Base64Error(e))?;

        // Derive key from password
        let session = derive_master_key(password, &salt)?;

        // Verify password
        if !verify_key(&session, &config.auth_verification_hash)? {
            return Err(VaultError::InvalidPassword);
        }

        // Load encrypted data
        let encrypted_blob = std::fs::read(&self.data_path)?;

        // Decrypt
        let decrypted = decrypt_blob(&encrypted_blob, &session)?;

        // Deserialize with Bincode
        let vfs_state: VfsState = bincode::deserialize(&decrypted)
            .map_err(|e| VaultError::Serialization(format!("Bincode deserialization failed: {}", e)))?;

        // Update status to Unlocked
        *status_guard = VfsStatus::Unlocked {
            fs: vfs_state,
            session,
        };

        Ok(())
    }

    /// Lock vault (zeroize keys)
    pub fn lock_vault(&self) -> Result<(), crate::api::vault_error::VaultError> {
        use crate::api::security::*;
        use crate::api::vault_error::VaultError;

        let mut status_guard = self.vault_status.write()
            .map_err(|_| VaultError::CryptoError("Lock poisoned".into()))?;

        // Check if unlocked
        if !matches!(*status_guard, VfsStatus::Unlocked { .. }) {
            return Ok(()); // Already locked, nothing to do
        }

        // Save current state before locking
        if let VfsStatus::Unlocked { ref fs, ref session } = *status_guard {
            // Serialize
            let serialized = bincode::serialize(fs)
                .map_err(|e| VaultError::Serialization(format!("Bincode serialization failed: {}", e)))?;

            // Encrypt
            let encrypted_blob = encrypt_blob(&serialized, session)?;

            // Save atomically
            atomic_write(&self.data_path, &encrypted_blob)?;
        }

        // Change status to Locked (this will drop session and trigger Zeroize)
        *status_guard = VfsStatus::Locked;

        Ok(())
    }

    // ========== Recovery Methods ==========

    /// Setup recovery for vault
    pub fn setup_recovery(
        &self,
        channels: Vec<crate::api::notification_channels::ChannelConfig>,
    ) -> Result<String, crate::api::vault_error::VaultError> {
        use crate::api::security::*;
        use crate::api::vault_error::VaultError;
        use crate::api::recovery::RecoveryManager;

        // Get current unlocked session
        let status_guard = self.vault_status.read()
            .map_err(|_| VaultError::CryptoError("Lock poisoned".into()))?;

        let session = match *status_guard {
            VfsStatus::Unlocked { ref session, .. } => session.clone(),
            _ => return Err(VaultError::Locked),
        };

        // Setup recovery and get recovery key + config
        let (recovery_key, recovery_config) = RecoveryManager::setup_recovery(&session, channels)?;
        println!("[VFS] Recovery key and config generated successfully");

        // Load existing vault config
        let mut config = load_vault_config(&self.config_path)?;
        println!("[VFS] Loaded existing config from: {:?}", self.config_path);

        // Update with recovery config
        config.recovery = Some(recovery_config);
        println!("[VFS] Updated config with recovery configuration");

        // Save updated config
        save_vault_config(&self.config_path, &config)?;
        println!("[VFS] Saved updated config with recovery to: {:?}", self.config_path);

        // Return recovery key mnemonic for user to save
        Ok(recovery_key.to_mnemonic())
    }

    /// Request password reset
    pub fn request_password_reset(
        &self,
        channel_type: &str,
    ) -> Result<(), crate::api::vault_error::VaultError> {
        use crate::api::security::*;
        use crate::api::vault_error::VaultError;
        use crate::api::recovery::RecoveryManager;

        // Load vault config
        let config = load_vault_config(&self.config_path)?;

        // Get recovery config
        let recovery_config = config.recovery
            .ok_or_else(|| VaultError::CryptoError("Recovery not configured".into()))?;

        // Create recovery session and send code
        let session = RecoveryManager::initiate_recovery(&recovery_config, channel_type)?;

        // Store session globally (we'll use a thread-local for now)
        RECOVERY_SESSION.with(|cell| {
            *cell.borrow_mut() = Some(session);
        });

        Ok(())
    }

    /// Complete password reset
    pub fn complete_password_reset(
        &self,
        code: &str,
        new_password: &str,
    ) -> Result<(), crate::api::vault_error::VaultError> {
        use crate::api::security::*;
        use crate::api::vault_error::VaultError;
        use crate::api::recovery::RecoveryManager;

        // Get recovery session
        let session = RECOVERY_SESSION.with(|cell| {
            cell.borrow_mut().take()
        }).ok_or_else(|| VaultError::CryptoError("No active recovery session".into()))?;

        // Verify code and decrypt recovery key
        let _recovery_key = session.verify_and_decrypt(code)?;

        // Load vault config to get salt
        let config = load_vault_config(&self.config_path)?;
        let salt = ::base64::engine::general_purpose::STANDARD.decode(&config.kdf_salt)
            .map_err(|e| VaultError::Base64Error(e))?;

        // Get recovery config
        let recovery_config = config.recovery
            .ok_or_else(|| VaultError::CryptoError("Recovery not configured".into()))?;

        // Generate new master key from new password
        let (_new_session, new_hash) = RecoveryManager::complete_recovery(
            &_recovery_key,
            new_password,
            &salt,
        )?;

        // TODO: Implement proper recovery key encryption of vault data
        // Current limitation: This resets the vault with new password
        // but existing encrypted data remains inaccessible without old password.
        // Future: Store vault data encrypted with both master key AND recovery key

        // Update config with new verification hash
        let mut new_config = VaultConfig::new(
            config.kdf_salt.clone(),
            new_hash,
        );
        new_config.recovery = Some(recovery_config);
        save_vault_config(&self.config_path, &new_config)?;

        // Update vault status to locked
        let mut status_guard = self.vault_status.write()
            .map_err(|_| VaultError::CryptoError("Lock poisoned".into()))?;
        *status_guard = VfsStatus::Locked;

        Ok(())
    }

    /// Get recovery channels
    pub fn get_recovery_channels(&self) -> Result<Vec<String>, crate::api::vault_error::VaultError> {
        use crate::api::security::*;
        use crate::api::vault_error::VaultError;

        // Load vault config
        println!("[VFS] Loading config from: {:?}", self.config_path);
        let config = load_vault_config(&self.config_path)?;
        println!("[VFS] Config loaded successfully, recovery present: {}", config.recovery.is_some());

        // Get recovery config
        let recovery_config = config.recovery
            .ok_or_else(|| {
                println!("[VFS] Recovery config is None!");
                VaultError::CryptoError("Recovery not configured".into())
            })?;

        println!("[VFS] Recovery config has {} channels", recovery_config.channels.len());

        // Extract channel types
        let channels: Vec<String> = recovery_config.channels
            .iter()
            .map(|c| format!("{:?}", c.channel_type()))
            .collect();

        println!("[VFS] Returning channels: {:?}", channels);
        Ok(channels)
    }

    /// Check if vault is unlocked
    fn check_vault_unlocked(&self) -> Result<(), crate::api::vault_error::VaultError> {
        use crate::api::security::*;
        use crate::api::vault_error::VaultError;

        if !self.vault_enabled {
            return Ok(()); // Vault disabled, pass through
        }

        let status_guard = self.vault_status.read()
            .map_err(|_| VaultError::CryptoError("Lock poisoned".into()))?;

        match *status_guard {
            VfsStatus::Unlocked { .. } => Ok(()),
            VfsStatus::Locked => Err(VaultError::Locked),
            VfsStatus::NotInitialized => Err(VaultError::NotInitialized),
        }
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

    fn create_file(&self, path: &str, name: &str, content: Option<&str>) -> FileSystemResult<()> {
        if name.contains('/') {
            return Err(FileSystemError::new("Имя файла не должно содержать '/'"));
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
                    return Err(FileSystemError::new(format!("Файл '{}' уже существует", name)));
                }

                let file_content = content
                    .map(|c| c.as_bytes().to_vec())
                    .unwrap_or_else(Vec::new);

                children.insert(name.to_string(), VfsNode::File {
                    content: file_content,
                    modified: current_timestamp(),
                    created: current_timestamp(),
                });
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

    fn create_files_batch(
        &self,
        path: &str,
        files: &[(String, Option<String>)],
    ) -> FileSystemResult<Vec<FileSystemResult<()>>> {
        let mut results = Vec::new();

        for (name, content) in files {
            let result = self.create_file(path, name, content.as_deref());
            results.push(result);
        }

        Ok(results)
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
                        Ok(::base64::engine::general_purpose::STANDARD.encode(&content))
                    }
                }
            }
            VfsNode::Directory { .. } => {
                Err(FileSystemError::new(format!("'{}' является директорией", path)))
            }
        }
    }

    fn write_file_content(&self, path: &str, content: &str) -> FileSystemResult<()> {
        let normalized = self.normalize_path_internal(path);

        let mut state = self.state.write()
            .map_err(|_| FileSystemError::new("Не удалось получить блокировку"))?;

        // Parse path to get parent directory and file name
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return Err(FileSystemError::new("Некорректный путь файла"));
        }

        let file_name = parts.last().unwrap();
        let parent_parts = &parts[..parts.len().saturating_sub(1)];

        // Navigate to parent directory
        let mut current = &mut state.root;
        for part in parent_parts {
            match current {
                VfsNode::Directory { children, .. } => {
                    current = children.get_mut(&part.to_string())
                        .ok_or_else(|| FileSystemError::new(format!("Директория не найдена: {}", part)))?;
                }
                VfsNode::File { .. } => {
                    return Err(FileSystemError::new("Путь содержит файл вместо директории"));
                }
            }
        }

        // Write file content (create or update)
        match current {
            VfsNode::Directory { children, modified, .. } => {
                // Check if file exists and update, otherwise create new
                if let Some(existing_node) = children.get_mut(*file_name) {
                    match existing_node {
                        VfsNode::File { content: existing_content, modified: file_modified, .. } => {
                            *existing_content = content.as_bytes().to_vec();
                            *file_modified = current_timestamp();
                        }
                        VfsNode::Directory { .. } => {
                            return Err(FileSystemError::new("Путь указывает на директорию"));
                        }
                    }
                } else {
                    // Create new file
                    children.insert(file_name.to_string(), VfsNode::File {
                        content: content.as_bytes().to_vec(),
                        modified: current_timestamp(),
                        created: current_timestamp(),
                    });
                }
                *modified = current_timestamp();

                drop(state);
                self.save_state()?;
                Ok(())
            }
            VfsNode::File { .. } => {
                Err(FileSystemError::new("Родительский путь не является директорией"))
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