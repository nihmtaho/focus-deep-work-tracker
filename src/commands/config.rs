//! `focus config` subcommand — get and set persistent configuration.
//!
//! Usage:
//!   focus config get theme
//!   focus config set theme onedark
//!   focus config set theme auto

use anyhow::{bail, Result};
use colored::Colorize;

use crate::config::{config_file_path, load_config, save_config};
use crate::theme::Theme;

/// Supported config keys.
const VALID_KEYS: &[&str] = &["theme", "vim-mode"];

pub fn run_get(key: &str) -> Result<()> {
    let path = config_file_path();
    let cfg = load_config(&path);

    match key {
        "theme" => {
            let value = cfg.theme.as_deref().unwrap_or("auto");
            println!("theme = {}", value.cyan());
        }
        "vim-mode" => {
            let value = if cfg.vim_mode { "on" } else { "off" };
            println!("vim-mode = {}", value.cyan());
        }
        _ => {
            bail!(
                "Unknown config key '{key}'. Valid keys: {}",
                VALID_KEYS.join(", ")
            );
        }
    }

    Ok(())
}

pub fn run_set(key: &str, value: &str) -> Result<()> {
    let path = config_file_path();
    let mut cfg = load_config(&path);

    match key {
        "theme" => {
            if value.eq_ignore_ascii_case("auto") {
                cfg.theme = None;
                println!("{}", "Theme set to auto (system detection)".green());
            } else if Theme::from_name(value).is_some() {
                cfg.theme = Some(value.to_lowercase());
                println!(
                    "{}",
                    format!("Theme set to '{}'", value.to_lowercase()).green()
                );
            } else {
                bail!(
                    "Unknown theme '{value}'. Valid themes: onedark, material, light, dark, auto"
                );
            }
        }
        "vim-mode" => {
            match value.to_lowercase().as_str() {
                "on" | "true" | "1" | "yes" => cfg.vim_mode = true,
                "off" | "false" | "0" | "no" => cfg.vim_mode = false,
                _ => bail!("Invalid value for vim-mode. Use: on/off"),
            }
            let status = if cfg.vim_mode { "enabled" } else { "disabled" };
            println!("{}", format!("Vim mode {status}").green());
        }
        _ => {
            bail!(
                "Unknown config key '{key}'. Valid keys: {}",
                VALID_KEYS.join(", ")
            );
        }
    }

    save_config(&path, &cfg)?;
    Ok(())
}
