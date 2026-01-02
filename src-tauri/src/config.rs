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
    pub created_at: u64, // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TabState {
    pub id: u64,
    pub path: Vec<String>,
    pub name: String,
}

// Режим отображения панелей
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PanelMode {
    Single,
    Dual,
}

impl Default for PanelMode {
    fn default() -> Self {
        PanelMode::Single
    }
}

// Состояние одной панели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    pub tabs: Vec<TabState>,
    pub active_tab_id: Option<u64>,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            tabs: vec![],
            active_tab_id: None,
        }
    }
}

// Конфигурация dual-panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualPanelConfig {
    #[serde(default = "default_panel_split")]
    pub left_panel_width_percent: u32,

    #[serde(default)]
    pub left_panel: PanelState,

    #[serde(default)]
    pub right_panel: PanelState,

    #[serde(default = "default_active_panel")]
    pub active_panel: String,
}

fn default_panel_split() -> u32 {
    50
}

fn default_active_panel() -> String {
    "left".to_string()
}

impl Default for DualPanelConfig {
    fn default() -> Self {
        Self {
            left_panel_width_percent: 50,
            left_panel: PanelState::default(),
            right_panel: PanelState::default(),
            active_panel: "left".to_string(),
        }
    }
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

    // Single-mode state
    #[serde(default)]
    pub tabs: Vec<TabState>,

    #[serde(default)]
    pub active_tab_id: Option<u64>,

    #[serde(default)]
    pub last_path: Option<Vec<String>>,

    // Dual-panel state
    #[serde(default)]
    pub panel_mode: PanelMode,

    #[serde(default)]
    pub dual_panel_config: DualPanelConfig,

    #[serde(default)]
    pub window: WindowState,

    #[serde(default)]
    pub sidebar: SidebarState,

    #[serde(default)]
    pub edit_mode_enabled: Option<bool>,
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
            panel_mode: PanelMode::default(),
            dual_panel_config: DualPanelConfig::default(),
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
            bookmarks: vec![],
            ui_state: Default::default(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.filesystem_backend, deserialized.filesystem_backend);
        assert_eq!(config.show_hidden_files, deserialized.show_hidden_files);
    }
}
