use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileSystemBackend {
    Real,
    Virtual,
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
        }
    }
}

impl AppConfig {
    /// Получить путь к файлу конфигурации
    pub fn config_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| "Could not determine config directory".to_string())?;

        let app_config_dir = config_dir.join("vfdir");

        // Создать директорию если её нет
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        Ok(app_config_dir.join("config.json"))
    }

    /// Загрузить конфигурацию из файла
    pub fn load() -> Result<Self, String> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Если файла нет, создаем дефолтный конфиг
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

    /// Сохранить конфигурацию в файл
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path()?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.filesystem_backend, FileSystemBackend::Real);
        assert_eq!(config.show_hidden_files, false);
        assert_eq!(config.default_view_mode, "grid");
        assert_eq!(config.theme, "luna");
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            filesystem_backend: FileSystemBackend::Virtual,
            show_hidden_files: true,
            default_view_mode: "list".to_string(),
            theme: "dark".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.filesystem_backend, deserialized.filesystem_backend);
        assert_eq!(config.show_hidden_files, deserialized.show_hidden_files);
    }
}
