//! Integration tests for theme loading and application
//!
//! Tests theme system initialization, loading, and application to UI.

use focus::theme::Theme;

#[test]
fn test_all_themes_load_successfully() {
    let themes = vec![Theme::OneDark, Theme::Material, Theme::Light, Theme::Dark];
    for theme in themes {
        let colors = theme.colors();
        assert_eq!(colors.validate(), Ok(()));
    }
}
