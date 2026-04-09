use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub vim_mode: bool,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub keyboard: KeyboardConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyboardConfig {
    #[serde(default = "default_true")]
    pub enable_number_shortcuts: bool,
    #[serde(default = "default_true")]
    pub enable_letter_shortcuts: bool,
}

fn default_true() -> bool {
    true
}

/// Returns the path to the config file: `{config_dir}/focus/config.json`
pub fn config_file_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("focus")
        .join("config.json")
}

/// Load config from `path`. Returns `AppConfig::default()` silently if file is
/// missing or malformed.
pub fn load_config(path: &Path) -> AppConfig {
    let Ok(data) = std::fs::read_to_string(path) else {
        return AppConfig::default();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

/// Persist config to `path`, creating parent directories as needed.
pub fn save_config(path: &Path, cfg: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cfg)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // T007: AppConfig tests
    #[test]
    fn default_vim_mode_is_false() {
        let cfg = AppConfig::default();
        assert!(!cfg.vim_mode);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let f = NamedTempFile::new().unwrap();
        let cfg = AppConfig { vim_mode: true };
        save_config(f.path(), &cfg).unwrap();
        let loaded = load_config(f.path());
        assert!(loaded.vim_mode);
    }

    #[test]
    fn load_config_missing_file_returns_default() {
        let cfg = load_config(Path::new("/nonexistent/path/config.json"));
        assert!(!cfg.vim_mode);
    }

    #[test]
    fn load_config_malformed_json_returns_default() {
        let f = NamedTempFile::new().unwrap();
        std::fs::write(f.path(), b"not valid json {{{{").unwrap();
        let cfg = load_config(f.path());
        assert!(!cfg.vim_mode);
    }

    #[test]
    fn save_config_writes_valid_json() {
        let f = NamedTempFile::new().unwrap();
        let cfg = AppConfig { vim_mode: false };
        save_config(f.path(), &cfg).unwrap();
        let raw = std::fs::read_to_string(f.path()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["vim_mode"], false);
    }
}
