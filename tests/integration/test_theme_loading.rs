//! Integration tests for theme loading and application
//!
//! Tests theme system initialization, loading, and application to UI.
//! Includes NO_COLOR verification per Constitution Principle IV.

use focus::theme::Theme;

#[test]
fn test_all_themes_load_successfully() {
    let themes = vec![Theme::OneDark, Theme::Material, Theme::Light, Theme::Dark];
    for theme in themes {
        let colors = theme.colors();
        assert_eq!(colors.validate(), Ok(()));
    }
}

/// Constitution Principle IV — T079:
/// The `colored` crate MUST produce plain text (no ANSI escape codes) when
/// color output is forced off. This simulates `NO_COLOR=1` behavior.
#[test]
fn test_no_color_colored_crate_produces_plain_text() {
    use colored::Colorize;

    // Force colored off — equivalent to what NO_COLOR=1 does at runtime
    colored::control::set_override(false);

    let output = "focus session".red().bold().to_string();

    colored::control::unset_override();

    assert!(
        !output.contains('\x1b'),
        "Expected no ANSI escape codes when NO_COLOR is active, but got: {:?}",
        output
    );
}

/// Constitution Principle IV — T146:
/// The TUI reads `std::env::var("NO_COLOR").is_ok()` to detect NO_COLOR.
/// Verify this pattern correctly detects presence (any value) and absence.
#[test]
fn test_no_color_env_var_detection_logic() {
    // Save current state
    let saved = std::env::var("NO_COLOR").ok();

    // Any non-empty value → detected
    std::env::set_var("NO_COLOR", "1");
    assert!(
        std::env::var("NO_COLOR").is_ok(),
        "NO_COLOR='1' should be detected as present"
    );

    // Empty string still counts as set (per NO_COLOR spec: any value)
    std::env::set_var("NO_COLOR", "");
    assert!(
        std::env::var("NO_COLOR").is_ok(),
        "NO_COLOR='' should still be detected (any value disables color)"
    );

    // Absent → not detected
    std::env::remove_var("NO_COLOR");
    assert!(
        std::env::var("NO_COLOR").is_err(),
        "no_color should be false when NO_COLOR env var is absent"
    );

    // Restore
    match saved {
        Some(v) => std::env::set_var("NO_COLOR", v),
        None => std::env::remove_var("NO_COLOR"),
    }
}
