//! Config module - Application configuration and settings

use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Theme name
    pub theme: String,
    /// Font family
    pub font_family: String,
    /// Font size in pixels
    pub font_size: f32,
    /// Tab width in spaces
    pub tab_width: usize,
    /// Whether to show line numbers
    pub show_line_numbers: bool,
    /// Whether to show minimap
    pub show_minimap: bool,
    /// Whether to enable auto-save
    pub auto_save: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
    /// Wallpaper path
    pub wallpaper_path: Option<PathBuf>,
    /// Whether wallpaper is enabled
    pub wallpaper_enabled: bool,
    /// Music directory path
    pub music_directory: Option<PathBuf>,
    /// Whether music player is enabled
    pub music_enabled: bool,
    /// Editor behavior settings
    pub editor: EditorConfig,
    /// UI settings
    pub ui: UISettings,
}

/// Editor-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Whether to use soft tabs (spaces instead of tabs)
    pub soft_tabs: bool,
    /// Whether to highlight current line
    pub highlight_current_line: bool,
    /// Whether to show matching brackets
    pub match_brackets: bool,
    /// Whether to auto-close brackets
    pub auto_close_brackets: bool,
    /// Whether to auto-close quotes
    pub auto_close_quotes: bool,
    /// Whether to show whitespace characters
    pub show_whitespace: bool,
    /// Whether to show indent guides
    pub show_indent_guides: bool,
    /// Maximum line length for word wrap
    pub word_wrap_column: usize,
    /// Whether to enable word wrap
    pub word_wrap: bool,
    /// Cursor style
    pub cursor_style: CursorStyle,
    /// Scroll speed
    pub scroll_speed: f32,
}

/// Cursor style options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CursorStyle {
    /// Block cursor █
    Block,
    /// Underline cursor _
    Underline,
    /// Beam cursor |
    Beam,
}

/// UI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
    /// Whether window is maximized
    pub window_maximized: bool,
    /// Sidebar width
    pub sidebar_width: u32,
    /// Status bar height
    pub status_bar_height: u32,
    /// Panel height (for music/timeline panels)
    pub panel_height: u32,
    /// UI scale factor
    pub scale_factor: f32,
    /// Whether to use native title bar
    pub native_title_bar: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            font_family: "JetBrains Mono".to_string(),
            font_size: 14.0,
            tab_width: 4,
            show_line_numbers: true,
            show_minimap: true,
            auto_save: false,
            auto_save_interval: 60,
            wallpaper_path: None,
            wallpaper_enabled: true,
            music_directory: None,
            music_enabled: true,
            editor: EditorConfig::default(),
            ui: UISettings::default(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            soft_tabs: true,
            highlight_current_line: true,
            match_brackets: true,
            auto_close_brackets: true,
            auto_close_quotes: true,
            show_whitespace: false,
            show_indent_guides: true,
            word_wrap_column: 120,
            word_wrap: false,
            cursor_style: CursorStyle::Beam,
            scroll_speed: 1.0,
        }
    }
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            window_maximized: false,
            sidebar_width: 250,
            status_bar_height: 24,
            panel_height: 200,
            scale_factor: 1.0,
            native_title_bar: true,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path();
        
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path();
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        
        Ok(())
    }

    /// Get the configuration file path
    pub fn config_file_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aurora_ide")
            .join("config.json")
    }

    /// Get the data directory
    pub fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aurora_ide")
    }

    /// Get the cache directory
    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aurora_ide")
    }
}
