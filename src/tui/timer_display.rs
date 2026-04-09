//! Timer Display Module
//!
//! Provides a digital flip-clock style timer display component for rendering
//! both Pomodoro and freeform session timers in HH:MM:SS or HHH:MM:SS format.

use std::time::Duration;

/// TimerDisplay component for rendering timer in flip-clock format
///
/// Supports both HH:MM:SS format (for durations < 100 hours) and
/// HHH:MM:SS format (for durations >= 100 hours up to 999:59:59).
#[derive(Debug, Clone)]
pub struct TimerDisplay {
    pub duration: Duration,
}

impl TimerDisplay {
    /// Create a new TimerDisplay with the given duration
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }

    /// Render the timer to a formatted string in HH:MM:SS or HHH:MM:SS format
    pub fn render(&self) -> String {
        let total_secs = self.duration.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        if hours >= 100 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        }
    }

    /// Check if this duration requires HHH:MM:SS format (>= 100 hours)
    pub fn requires_extended_format(&self) -> bool {
        self.duration.as_secs() >= 360000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_format_hh_mm_ss_under_100_hours() {
        let display = TimerDisplay::new(Duration::from_secs(3661)); // 1:01:01
        assert_eq!(display.render(), "01:01:01");
    }

    #[test]
    fn test_render_format_hh_mm_ss_zero() {
        let display = TimerDisplay::new(Duration::from_secs(0));
        assert_eq!(display.render(), "00:00:00");
    }

    #[test]
    fn test_render_format_hh_mm_ss_boundary_99_59_59() {
        let display = TimerDisplay::new(Duration::from_secs(359999)); // 99:59:59
        assert_eq!(display.render(), "99:59:59");
    }

    #[test]
    fn test_render_format_hhh_mm_ss_100_hours() {
        let display = TimerDisplay::new(Duration::from_secs(360000)); // 100:00:00
        assert_eq!(display.render(), "100:00:00");
    }

    #[test]
    fn test_requires_extended_format_under_100() {
        let display = TimerDisplay::new(Duration::from_secs(359999));
        assert!(!display.requires_extended_format());
    }

    #[test]
    fn test_requires_extended_format_over_100() {
        let display = TimerDisplay::new(Duration::from_secs(360000));
        assert!(display.requires_extended_format());
    }
}
