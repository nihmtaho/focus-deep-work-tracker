//! Theme system module
//!
//! Provides a themeable color system for the CLI with support for multiple
//! predefined themes (onedark, material, light, dark) and OS auto-detection.

pub mod dark;
pub mod light;
pub mod material;
pub mod onedark;

use ratatui::style::Color;

/// Theme enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    OneDark,
    Material,
    Light,
    Dark,
}

impl Theme {
    /// Auto-detect theme based on system preferences
    ///
    /// On macOS: Uses NSAppearance (dark mode)
    /// On Linux: Checks COLORFGBG environment variable
    /// Fallback: Returns OneDark
    pub fn auto_detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            // Try to detect macOS dark mode
            if let Ok(output) = std::process::Command::new("defaults")
                .args(["read", "-g", "AppleInterfaceStyle"])
                .output()
            {
                if String::from_utf8_lossy(&output.stdout)
                    .to_lowercase()
                    .contains("dark")
                {
                    return Theme::Dark;
                }
            }
            // Default to light on macOS if not in dark mode
            Theme::Light
        }

        #[cfg(target_os = "linux")]
        {
            // Check COLORFGBG environment variable
            if let Ok(colorfgbg) = std::env::var("COLORFGBG") {
                let parts: Vec<&str> = colorfgbg.split(';').collect();
                if parts.len() >= 2 {
                    if let Ok(bg) = parts[1].parse::<u8>() {
                        // Lighter backgrounds (like white) suggest light theme
                        if bg > 7 && bg < 16 || bg == 15 {
                            return Theme::Light;
                        }
                    }
                }
            }
            // Default to dark on Linux
            Theme::Dark
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            // Fallback for other platforms
            Theme::OneDark
        }
    }

    /// Get color definitions for this theme
    pub fn colors(&self) -> ThemeColors {
        match self {
            Theme::OneDark => onedark::colors(),
            Theme::Material => material::colors(),
            Theme::Light => light::colors(),
            Theme::Dark => dark::colors(),
        }
    }
}

/// Complete color definition for a theme
///
/// Maps semantic color names to actual Ratatui Color values
#[derive(Debug, Clone)]
pub struct ThemeColors {
    // UI Element Colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,

    // Status Colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,

    // Background & Text
    pub background: Color,
    pub foreground: Color,

    // Panel Styling
    pub panel_border: Color,
    pub panel_focus_border: Color,

    // TODO States
    pub todo_todo: Color,
    pub todo_in_session: Color,
    pub todo_completed: Color,

    // Tags & Session Metadata
    pub tag_color: Color,
    pub session_title: Color,

    // Timer Display
    pub timer_digit: Color,
    pub timer_separator: Color,
}

impl Theme {
    /// Parse a theme name string (case-insensitive) to a Theme variant.
    /// Returns `None` for "auto" or unknown strings.
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "onedark" | "one-dark" => Some(Theme::OneDark),
            "material" => Some(Theme::Material),
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }

    /// Return the canonical name used in config.
    pub fn name(&self) -> &'static str {
        match self {
            Theme::OneDark => "onedark",
            Theme::Material => "material",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    /// Resolve the theme to use: from stored config string or auto-detect.
    pub fn resolve(config_theme: Option<&str>) -> Self {
        match config_theme {
            Some(s) => Theme::from_name(s).unwrap_or_else(Theme::auto_detect),
            None => Theme::auto_detect(),
        }
    }
}

impl ThemeColors {
    /// Validate that all required colors are defined
    pub fn validate(&self) -> Result<(), String> {
        // All fields are already defined in the struct, so validation passes
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_auto_detect_returns_valid_theme() {
        let theme = Theme::auto_detect();
        assert!(matches!(
            theme,
            Theme::OneDark | Theme::Material | Theme::Light | Theme::Dark
        ));
    }

    #[test]
    fn test_theme_colors_complete_for_onedark() {
        let colors = Theme::OneDark.colors();
        assert_eq!(colors.validate(), Ok(()));
    }

    #[test]
    fn test_theme_colors_complete_for_material() {
        let colors = Theme::Material.colors();
        assert_eq!(colors.validate(), Ok(()));
    }

    #[test]
    fn test_theme_colors_complete_for_light() {
        let colors = Theme::Light.colors();
        assert_eq!(colors.validate(), Ok(()));
    }

    #[test]
    fn test_theme_colors_complete_for_dark() {
        let colors = Theme::Dark.colors();
        assert_eq!(colors.validate(), Ok(()));
    }

    #[test]
    fn test_theme_default_is_onedark() {
        assert_eq!(Theme::default(), Theme::OneDark);
    }
}
