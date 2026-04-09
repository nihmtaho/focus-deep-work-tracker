//! Theme system for UI components
//!
//! Provides utilities for loading and applying themes to UI elements.

use crate::theme::{Theme, ThemeColors};

/// Load a theme by name with fallback behavior
pub fn load_theme(theme_name: Option<&str>) -> Theme {
    match theme_name {
        Some("onedark") => Theme::OneDark,
        Some("material") => Theme::Material,
        Some("light") => Theme::Light,
        Some("dark") => Theme::Dark,
        Some("auto") | None => Theme::auto_detect(),
        Some(_) => {
            // Invalid theme name, fallback to auto-detect
            Theme::auto_detect()
        }
    }
}

/// Get the current theme colors (cached)
pub fn get_current_colors() -> ThemeColors {
    // TODO: Integrate with config system to read saved theme
    // For now, use auto-detection
    Theme::auto_detect().colors()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_theme_onedark() {
        let theme = load_theme(Some("onedark"));
        assert_eq!(theme, Theme::OneDark);
    }

    #[test]
    fn test_load_theme_material() {
        let theme = load_theme(Some("material"));
        assert_eq!(theme, Theme::Material);
    }

    #[test]
    fn test_load_theme_light() {
        let theme = load_theme(Some("light"));
        assert_eq!(theme, Theme::Light);
    }

    #[test]
    fn test_load_theme_dark() {
        let theme = load_theme(Some("dark"));
        assert_eq!(theme, Theme::Dark);
    }

    #[test]
    fn test_load_theme_auto_detection() {
        let theme = load_theme(Some("auto"));
        assert!(matches!(theme, Theme::OneDark | Theme::Material | Theme::Light | Theme::Dark));
    }

    #[test]
    fn test_load_theme_invalid_fallback_to_auto() {
        let theme = load_theme(Some("invalid_theme"));
        assert!(matches!(theme, Theme::OneDark | Theme::Material | Theme::Light | Theme::Dark));
    }

    #[test]
    fn test_get_current_colors_returns_valid_colors() {
        let colors = get_current_colors();
        assert_eq!(colors.validate(), Ok(()));
    }
}
