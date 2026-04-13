//! Material Design theme colors
//!
//! A modern theme based on Google's Material Design color palette.
//! Balanced, accessible colors suitable for all users.

use super::ThemeColors;
use ratatui::style::Color;

/// Get Material Design theme colors
pub fn colors() -> ThemeColors {
    ThemeColors {
        // UI Elements
        primary: Color::Rgb(66, 165, 245),    // Blue
        secondary: Color::Rgb(102, 187, 106), // Green
        accent: Color::Rgb(171, 71, 188),     // Purple

        // Status Colors
        success: Color::Rgb(102, 187, 106), // Green
        warning: Color::Rgb(255, 167, 38),  // Orange
        error: Color::Rgb(239, 83, 80),     // Red

        // Background & Text
        background: Color::Rgb(38, 50, 56),    // Dark background
        foreground: Color::Rgb(236, 239, 241), // Light text

        // Panel Styling
        panel_border: Color::Rgb(69, 90, 100), // Dark gray
        panel_focus_border: Color::Rgb(66, 165, 245), // Bright blue

        // TODO States
        todo_todo: Color::Rgb(144, 164, 174), // Default gray
        todo_in_session: Color::Rgb(255, 167, 38), // Active orange
        todo_completed: Color::Rgb(102, 187, 106), // Completed green

        // Tags & Session Metadata
        tag_color: Color::Rgb(41, 182, 246),     // Light blue
        session_title: Color::Rgb(66, 165, 245), // Blue

        // Timer Display
        timer_digit: Color::Rgb(239, 83, 80),       // Red
        timer_separator: Color::Rgb(144, 164, 174), // Gray
    }
}
