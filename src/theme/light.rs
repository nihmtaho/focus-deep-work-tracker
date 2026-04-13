//! Light theme colors
//!
//! A light theme suitable for bright environments and light terminal backgrounds.
//! Designed for readability in daytime use.

use super::ThemeColors;
use ratatui::style::Color;

/// Get Light theme colors
pub fn colors() -> ThemeColors {
    ThemeColors {
        // UI Elements
        primary: Color::Rgb(2, 119, 189),   // Dark blue
        secondary: Color::Rgb(0, 137, 123), // Teal
        accent: Color::Rgb(123, 31, 162),   // Deep purple

        // Status Colors
        success: Color::Rgb(56, 142, 60), // Dark green
        warning: Color::Rgb(245, 127, 0), // Deep orange
        error: Color::Rgb(198, 40, 40),   // Dark red

        // Background & Text
        background: Color::Rgb(250, 250, 250), // Light background
        foreground: Color::Rgb(33, 33, 33),    // Dark text

        // Panel Styling
        panel_border: Color::Rgb(189, 189, 189), // Medium gray
        panel_focus_border: Color::Rgb(2, 119, 189), // Bright blue

        // TODO States
        todo_todo: Color::Rgb(97, 97, 97),        // Medium gray
        todo_in_session: Color::Rgb(245, 127, 0), // Active orange
        todo_completed: Color::Rgb(56, 142, 60),  // Completed green

        // Tags & Session Metadata
        tag_color: Color::Rgb(2, 136, 209),     // Light blue
        session_title: Color::Rgb(2, 119, 189), // Dark blue

        // Timer Display
        timer_digit: Color::Rgb(198, 40, 40),    // Dark red
        timer_separator: Color::Rgb(97, 97, 97), // Gray
    }
}
