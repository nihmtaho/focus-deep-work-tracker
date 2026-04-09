//! Dark theme colors
//!
//! A high-contrast dark theme with vibrant accent colors.
//! Designed for low-light environments and maximum contrast.

use ratatui::style::Color;
use super::ThemeColors;

/// Get Dark theme colors
pub fn colors() -> ThemeColors {
    ThemeColors {
        // UI Elements
        primary: Color::Rgb(187, 134, 252),      // Light purple
        secondary: Color::Rgb(3, 218, 198),      // Bright teal
        accent: Color::Rgb(207, 102, 121),       // Rose

        // Status Colors
        success: Color::Rgb(3, 218, 198),        // Bright teal
        warning: Color::Rgb(255, 179, 0),        // Bright orange
        error: Color::Rgb(207, 102, 121),        // Rose

        // Background & Text
        background: Color::Rgb(18, 18, 18),      // Pure black
        foreground: Color::Rgb(225, 225, 225),   // Off-white

        // Panel Styling
        panel_border: Color::Rgb(55, 55, 55),    // Dark gray
        panel_focus_border: Color::Rgb(187, 134, 252), // Bright purple

        // TODO States
        todo_todo: Color::Rgb(176, 176, 176),    // Light gray
        todo_in_session: Color::Rgb(255, 179, 0),    // Active orange
        todo_completed: Color::Rgb(3, 218, 198),     // Completed teal

        // Tags & Session Metadata
        tag_color: Color::Rgb(3, 218, 198),      // Bright teal
        session_title: Color::Rgb(187, 134, 252),     // Light purple

        // Timer Display
        timer_digit: Color::Rgb(207, 102, 121),  // Rose
        timer_separator: Color::Rgb(176, 176, 176),   // Light gray
    }
}
