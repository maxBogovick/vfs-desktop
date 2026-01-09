use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use crate::api::security::{VaultSession, encrypt_blob, decrypt_blob, atomic_write};
use crate::api::vault_error::{VaultResult, VaultError};

/// Менеджер зашифрованного хранилища контента
#[derive(Clone, Debug)]
pub struct BlobStore {
    base_path: PathBuf,
}

impl BlobStore {
    pub fn new(base_path: PathBuf) -> Self {
        // Создаем директорию если не существует
        if !base_path.exists() {
            let _ = fs::create_dir_all(&base_path);
        }
        Self { base_path }
    }

    /// Генерирует путь к файлу по ID
    fn get_path(&self, file_id: &str) -> PathBuf {
        self.base_path.join(file_id)
    }

    /// Сохраняет контент в файл и возвращает его ID (UUID)
    /// Если session есть - шифрует, иначе пишет как есть.
    pub fn write(&self, data: &[u8], session: Option<&VaultSession>, file_id: Option<&str>) -> VaultResult<String> {
        // Генерируем новый ID если не передан
        let id = file_id.map(String::from).unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let path = self.get_path(&id);

        let data_to_write = if let Some(sess) = session {
            encrypt_blob(data, sess)?
        } else {
            data.to_vec()
        };

        // Атомарно записываем на диск
        atomic_write(&path, &data_to_write)?;

        Ok(id)
    }

    /// Читает контент по ID
    /// Если session есть - пытается расшифровать.
    pub fn read(&self, file_id: &str, session: Option<&VaultSession>) -> VaultResult<Vec<u8>> {
        let path = self.get_path(file_id);

        if !path.exists() {
            return Err(VaultError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound, 
                format!("Blob {} not found", file_id)
            )));
        }

        let file_data = fs::read(&path)
            .map_err(|e| VaultError::Io(e))?;

        if let Some(sess) = session {
             decrypt_blob(&file_data, sess)
        } else {
             Ok(file_data)
        }
    }

    /// Удаляет файл контента
    pub fn delete(&self, file_id: &str) -> VaultResult<()> {
        let path = self.get_path(file_id);
        if path.exists() {
            fs::remove_file(path).map_err(|e| VaultError::Io(e))?;
        }
        Ok(())
    }
}
