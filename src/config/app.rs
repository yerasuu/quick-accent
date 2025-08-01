use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::screen::ScreenConfig;
use crate::config::tool::ToolConfig;
use crate::config::window::WindowConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub screen: ScreenConfig,
    /// Application text content
    pub tool: ToolConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            screen: ScreenConfig::default(),
            tool: ToolConfig::default(),
        }
    }
}

impl AppConfig {
    /// Get the default config file path
    pub fn default_config_path() -> PathBuf {
        // Try XDG config directory first, then fall back to home directory
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config)
                .join("quick-accent")
                .join("config.ron")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join("quick-accent")
                .join("config.ron")
        } else {
            // Final fallback to current directory
            PathBuf::from("config.ron")
        }
    }

    /// Load configuration from file, create default if not exists
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::default_config_path();
        Self::load_from_path(&config_path)
    }

    /// Load configuration from specific path
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();

        if !path.exists() {
            eprintln!(
                "Config file not found at {:?}, creating default config...",
                path
            );
            let default_config = Self::default();
            default_config.save_to_path(path)?;
            return Ok(default_config);
        }

        let config_content = fs::read_to_string(path)?;
        let config: AppConfig = ron::from_str(&config_content)?;
        eprintln!("Loaded config from {:?}", path);
        Ok(config)
    }

    /// Save configuration to default path
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::default_config_path();
        self.save_to_path(&config_path)
    }

    /// Save configuration to specific path
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        fs::write(path, config_content)?;
        eprintln!("Saved config to {:?}", path);
        Ok(())
    }

    /// Calculate window dimensions based on screen size and config
    pub fn calculate_window_size(&self, screen_width: f32, screen_height: f32) -> (f32, f32) {
        let window_width = screen_width * self.window.width_fraction;
        let window_height = self.window.height;
        (window_width, window_height)
    }

    /// Calculate window position based on screen size and config
    pub fn calculate_window_position(
        &self,
        screen_width: f32,
        screen_height: f32,
        window_width: f32,
    ) -> (f32, f32) {
        let x_position = if self.window.center_horizontally {
            (screen_width - window_width) / 2.0
        } else {
            self.window.x_offset
        };

        let y_position = screen_height * self.window.y_position_fraction;

        (x_position, y_position)
    }
}
