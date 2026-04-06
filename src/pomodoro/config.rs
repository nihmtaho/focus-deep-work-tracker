use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::error::FocusError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroConfig {
    pub work_duration_mins: u32,
    pub break_duration_mins: u32,
    pub long_break_duration_mins: u32,
    pub long_break_after: u32,
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_duration_mins: 25,
            break_duration_mins: 5,
            long_break_duration_mins: 15,
            long_break_after: 4,
        }
    }
}

/// Returns the path to `~/.config/focus/pomodoro.toml`.
pub fn pomodoro_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("focus")
        .join("pomodoro.toml")
}

/// Load config from a TOML file. Returns `PomodoroConfig::default()` silently
/// if the file is missing or malformed.
pub fn load_from_file(path: &Path) -> PomodoroConfig {
    let Ok(data) = std::fs::read_to_string(path) else {
        return PomodoroConfig::default();
    };
    toml::from_str(&data).unwrap_or_default()
}

impl PomodoroConfig {
    /// Override fields from `FOCUS_POMODORO_*` environment variables.
    pub fn apply_env(&mut self) {
        if let Ok(v) = std::env::var("FOCUS_POMODORO_WORK") {
            if let Ok(n) = v.parse::<u32>() {
                self.work_duration_mins = n;
            }
        }
        if let Ok(v) = std::env::var("FOCUS_POMODORO_BREAK") {
            if let Ok(n) = v.parse::<u32>() {
                self.break_duration_mins = n;
            }
        }
        if let Ok(v) = std::env::var("FOCUS_POMODORO_LONG_BREAK") {
            if let Ok(n) = v.parse::<u32>() {
                self.long_break_duration_mins = n;
            }
        }
        if let Ok(v) = std::env::var("FOCUS_POMODORO_LONG_BREAK_AFTER") {
            if let Ok(n) = v.parse::<u32>() {
                self.long_break_after = n;
            }
        }
    }

    /// Override fields from CLI flag values (None = not provided, keep current).
    pub fn apply_cli_flags(
        &mut self,
        work: Option<u32>,
        break_mins: Option<u32>,
        long_break: Option<u32>,
        after: Option<u32>,
    ) {
        if let Some(v) = work {
            self.work_duration_mins = v;
        }
        if let Some(v) = break_mins {
            self.break_duration_mins = v;
        }
        if let Some(v) = long_break {
            self.long_break_duration_mins = v;
        }
        if let Some(v) = after {
            self.long_break_after = v;
        }
    }

    /// Validate duration ranges. Returns an error with a clear message on failure.
    pub fn validate(&self) -> Result<()> {
        if self.work_duration_mins < 1 || self.work_duration_mins > 120 {
            return Err(FocusError::InvalidPomoDuration {
                field: "work".to_string(),
                value: self.work_duration_mins,
                min: 1,
                max: 120,
            }
            .into());
        }
        if self.break_duration_mins < 1 || self.break_duration_mins > 60 {
            return Err(FocusError::InvalidPomoDuration {
                field: "break".to_string(),
                value: self.break_duration_mins,
                min: 1,
                max: 60,
            }
            .into());
        }
        if self.long_break_duration_mins < 1 || self.long_break_duration_mins > 60 {
            return Err(FocusError::InvalidPomoDuration {
                field: "long-break".to_string(),
                value: self.long_break_duration_mins,
                min: 1,
                max: 60,
            }
            .into());
        }
        Ok(())
    }

    /// Build a fully-resolved config: file → env → CLI flags.
    pub fn resolve(
        work: Option<u32>,
        break_mins: Option<u32>,
        long_break: Option<u32>,
        after: Option<u32>,
    ) -> Result<Self> {
        let path = pomodoro_config_path();
        let mut cfg = load_from_file(&path);
        cfg.apply_env();
        cfg.apply_cli_flags(work, break_mins, long_break, after);
        cfg.validate()?;
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn default_values() {
        let cfg = PomodoroConfig::default();
        assert_eq!(cfg.work_duration_mins, 25);
        assert_eq!(cfg.break_duration_mins, 5);
        assert_eq!(cfg.long_break_duration_mins, 15);
        assert_eq!(cfg.long_break_after, 4);
    }

    #[test]
    fn load_from_file_custom_values() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "work_duration_mins = 45").unwrap();
        writeln!(f, "break_duration_mins = 10").unwrap();
        writeln!(f, "long_break_duration_mins = 20").unwrap();
        writeln!(f, "long_break_after = 3").unwrap();
        let cfg = load_from_file(f.path());
        assert_eq!(cfg.work_duration_mins, 45);
        assert_eq!(cfg.break_duration_mins, 10);
        assert_eq!(cfg.long_break_duration_mins, 20);
        assert_eq!(cfg.long_break_after, 3);
    }

    #[test]
    fn load_from_file_missing_returns_default() {
        let cfg = load_from_file(Path::new("/nonexistent/pomodoro.toml"));
        assert_eq!(cfg.work_duration_mins, 25);
    }

    #[test]
    fn load_from_file_malformed_returns_default() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "not valid toml !!!").unwrap();
        let cfg = load_from_file(f.path());
        assert_eq!(cfg.work_duration_mins, 25);
    }

    #[test]
    fn cli_flags_override_defaults() {
        let mut cfg = PomodoroConfig::default();
        cfg.apply_cli_flags(Some(45), Some(15), Some(20), Some(3));
        assert_eq!(cfg.work_duration_mins, 45);
        assert_eq!(cfg.break_duration_mins, 15);
        assert_eq!(cfg.long_break_duration_mins, 20);
        assert_eq!(cfg.long_break_after, 3);
    }

    #[test]
    fn cli_none_flags_keep_current_value() {
        let mut cfg = PomodoroConfig::default();
        cfg.apply_cli_flags(Some(30), None, None, None);
        assert_eq!(cfg.work_duration_mins, 30);
        assert_eq!(cfg.break_duration_mins, 5);
    }

    #[test]
    fn validate_rejects_work_zero() {
        let cfg = PomodoroConfig {
            work_duration_mins: 0,
            ..PomodoroConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("work"));
    }

    #[test]
    fn validate_rejects_work_too_large() {
        let cfg = PomodoroConfig {
            work_duration_mins: 121,
            ..PomodoroConfig::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_break_zero() {
        let cfg = PomodoroConfig {
            break_duration_mins: 0,
            ..PomodoroConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("break"));
    }

    #[test]
    fn validate_rejects_break_too_large() {
        let cfg = PomodoroConfig {
            break_duration_mins: 61,
            ..PomodoroConfig::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_long_break_zero() {
        let cfg = PomodoroConfig {
            long_break_duration_mins: 0,
            ..PomodoroConfig::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_long_break_too_large() {
        let cfg = PomodoroConfig {
            long_break_duration_mins: 61,
            ..PomodoroConfig::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_accepts_boundary_values() {
        let cfg = PomodoroConfig {
            work_duration_mins: 1,
            break_duration_mins: 1,
            long_break_duration_mins: 1,
            long_break_after: 2,
        };
        assert!(cfg.validate().is_ok());

        let cfg2 = PomodoroConfig {
            work_duration_mins: 120,
            break_duration_mins: 60,
            long_break_duration_mins: 60,
            long_break_after: 8,
        };
        assert!(cfg2.validate().is_ok());
    }
}
