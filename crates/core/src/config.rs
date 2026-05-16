use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub engines: Vec<EngineEntry>,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEntry {
    pub name: String,
    pub command: String,
    pub initial_commands: String,
    pub analyze_interval_cs: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub board_size: usize,
    pub show_coordinates: bool,
    pub show_move_numbers: bool,
    pub show_winrate_colors: bool,
    pub dark_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            engines: Vec::new(),
            ui: UiConfig {
                board_size: 19,
                show_coordinates: true,
                show_move_numbers: false,
                show_winrate_colors: true,
                dark_mode: true,
            },
        }
    }
}

impl AppConfig {
    /// Load config from the given path.
    /// If the file doesn't exist, creates a default config.
    /// If the file exists but has invalid JSON, returns default without overwriting.
    pub fn load(path: &PathBuf) -> Self {
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            match serde_json::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    log::warn!("Failed to parse config file: {}. Using defaults.", e);
                    AppConfig::default()
                }
            }
        } else {
            let config = AppConfig::default();
            // Try to save default config, but don't fail if it can't
            if config.save(path).is_err() {
                log::warn!("Failed to create default config file at {:?}", path);
            }
            config
        }
    }

    /// Save config to the given path.
    pub fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self).unwrap_or_default();
        fs::write(path, content)?;
        Ok(())
    }
}