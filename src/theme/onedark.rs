//! OneDark theme colors
//!
//! A dark theme inspired by Atom's OneDark color scheme.
//! High contrast, developer-friendly colors suitable for long coding sessions.

use super::ThemeColors;
use ratatui::style::Color;

/// Get OneDark theme colors
pub fn colors() -> ThemeColors {
    ThemeColors {
        // UI Elements
        primary: Color::Rgb(198, 120, 221),  // Purple
        secondary: Color::Rgb(97, 175, 239), // Blue
        accent: Color::Rgb(86, 182, 194),    // Cyan

        // Status Colors
        success: Color::Rgb(152, 195, 121), // Green
        warning: Color::Rgb(229, 192, 123), // Yellow
        error: Color::Rgb(224, 108, 117),   // Red

        // Background & Text
        background: Color::Rgb(40, 44, 52),    // Dark background
        foreground: Color::Rgb(171, 178, 191), // Light gray text

        // Panel Styling
        panel_border: Color::Rgb(62, 68, 82), // Dark gray
        panel_focus_border: Color::Rgb(97, 175, 239), // Bright blue

        // TODO States
        todo_todo: Color::Rgb(171, 178, 191), // Default gray
        todo_in_session: Color::Rgb(229, 192, 123), // Active yellow
        todo_completed: Color::Rgb(152, 195, 121), // Completed green

        // Tags & Session Metadata
        tag_color: Color::Rgb(86, 182, 194),      // Cyan
        session_title: Color::Rgb(198, 120, 221), // Purple

        // Timer Display
        timer_digit: Color::Rgb(224, 108, 117),     // Red
        timer_separator: Color::Rgb(171, 178, 191), // Gray
    }
}
