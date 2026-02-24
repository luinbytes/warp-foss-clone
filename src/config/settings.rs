//! Application settings loaded from config file

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Terminal settings
    #[serde(default)]
    pub terminal: TerminalConfig,
    /// Font settings
    #[serde(default)]
    pub font: FontConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            terminal: TerminalConfig::default(),
            font: FontConfig::default(),
        }
    }
}

/// Terminal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Shell command to spawn (defaults to $SHELL or /bin/sh)
    #[serde(default)]
    pub shell: Option<String>,
    /// Initial terminal width in columns
    #[serde(default = "default_cols")]
    pub cols: u16,
    /// Initial terminal height in rows
    #[serde(default = "default_rows")]
    pub rows: u16,
    /// Working directory for the shell (empty means current directory)
    #[serde(default)]
    pub working_dir: Option<String>,
    /// Environment variables to set
    #[serde(default)]
    pub env: Vec<(String, String)>,
}

fn default_cols() -> u16 {
    120
}

fn default_rows() -> u16 {
    40
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: None,
            cols: default_cols(),
            rows: default_rows(),
            working_dir: None,
            env: Vec::new(),
        }
    }
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Font size in points
    #[serde(default = "default_font_size")]
    pub size: f32,
}

fn default_font_size() -> f32 {
    14.0
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            size: default_font_size(),
        }
    }
}

impl Config {
    /// Load configuration from the default location
    ///
    /// Looks for config at:
    /// 1. ~/.config/warp-foss/config.toml
    /// 2. Creates default config if not found
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            Self::load_from_path(&config_path)
        } else {
            // Create default config
            let config = Self::default();
            config.save_to_path(&config_path)?;
            tracing::info!("Created default config at {:?}", config_path);
            Ok(config)
        }
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", path))
    }

    /// Save configuration to a specific path
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        std::fs::write(path, &contents)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }

    /// Get the default config file path
    ///
    /// Priority:
    /// 1. $XDG_CONFIG_HOME/warp-foss/config.toml
    /// 2. ~/.config/warp-foss/config.toml
    pub fn config_path() -> Result<PathBuf> {
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            Ok(PathBuf::from(xdg_config).join("warp-foss").join("config.toml"))
        } else {
            let home = dirs::home_dir()
                .context("Could not determine home directory")?;
            Ok(home.join(".config").join("warp-foss").join("config.toml"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.terminal.cols, 120);
        assert_eq!(config.terminal.rows, 40);
        assert_eq!(config.font.size, 14.0);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[terminal]"));
        assert!(toml_str.contains("[font]"));

        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.terminal.cols, config.terminal.cols);
        assert_eq!(parsed.font.size, config.font.size);
    }

    #[test]
    fn test_custom_config() {
        let toml_str = r#"
[terminal]
shell = "/bin/fish"
cols = 100
rows = 30
working_dir = "/home/user"

[font]
size = 16.0
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.terminal.shell, Some("/bin/fish".to_string()));
        assert_eq!(config.terminal.cols, 100);
        assert_eq!(config.terminal.rows, 30);
        assert_eq!(config.terminal.working_dir, Some("/home/user".to_string()));
        assert_eq!(config.font.size, 16.0);
    }

    #[test]
    fn test_partial_config() {
        let toml_str = r#"
[font]
size = 18.0
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        // Terminal should use defaults
        assert_eq!(config.terminal.cols, 120);
        assert_eq!(config.terminal.rows, 40);
        // Font should use custom value
        assert_eq!(config.font.size, 18.0);
    }
}
