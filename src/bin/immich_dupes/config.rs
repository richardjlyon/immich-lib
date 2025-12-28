//! Configuration file support for immich-dupes CLI.
//!
//! Provides persistent configuration storage in OS-native locations:
//! - macOS: ~/Library/Application Support/immich-dupes/config.toml
//! - Linux: ~/.config/immich-dupes/config.toml
//! - Windows: C:\Users\<user>\AppData\Roaming\immich-dupes\config.toml

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// Application configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Server connection settings.
    #[serde(default)]
    pub server: ServerConfig,
}

/// Server connection configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Immich server URL.
    pub url: Option<String>,
    /// API key for authentication.
    pub api_key: Option<String>,
}

/// Returns the path to the configuration file.
///
/// Uses OS-native configuration directories via the `directories` crate.
/// Falls back to `~/.config/immich-dupes/config.toml` if ProjectDirs fails.
pub fn config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "immich-dupes") {
        proj_dirs.config_dir().join("config.toml")
    } else {
        // Fallback for rare cases where ProjectDirs fails
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("immich-dupes")
            .join("config.toml")
    }
}

/// Loads configuration from the config file.
///
/// Returns `Config::default()` if the file doesn't exist or parsing fails.
/// This allows the application to work without a config file.
pub fn load() -> Config {
    load_inner().unwrap_or_default()
}

/// Internal load function that returns errors for debugging.
fn load_inner() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Saves configuration to the config file.
///
/// Creates parent directories if they don't exist.
/// Writes atomically by writing to a temp file then renaming.
#[allow(dead_code)]
pub fn save(config: &Config) -> Result<()> {
    let path = config_path();

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    // Serialize to TOML
    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

    // Write atomically: write to temp file, then rename
    let temp_path = path.with_extension("toml.tmp");

    let mut file = fs::File::create(&temp_path)
        .with_context(|| format!("Failed to create temp config file: {}", temp_path.display()))?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write config file: {}", temp_path.display()))?;

    file.sync_all()
        .with_context(|| format!("Failed to sync config file: {}", temp_path.display()))?;

    // Rename temp file to actual config file
    fs::rename(&temp_path, &path).with_context(|| {
        format!(
            "Failed to rename temp config {} to {}",
            temp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.server.url.is_none());
        assert!(config.server.api_key.is_none());
    }

    #[test]
    fn test_config_path_exists() {
        let path = config_path();
        // Path should end with config.toml
        assert!(path.ends_with("config.toml"));
        // Path should contain immich-dupes
        assert!(path.to_string_lossy().contains("immich-dupes"));
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        // Loading from a nonexistent path should return defaults
        let config = load();
        assert!(config.server.url.is_none());
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = Config {
            server: ServerConfig {
                url: Some("https://immich.example.com".to_string()),
                api_key: Some("test-api-key".to_string()),
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(
            parsed.server.url.as_deref(),
            Some("https://immich.example.com")
        );
        assert_eq!(parsed.server.api_key.as_deref(), Some("test-api-key"));
    }
}
