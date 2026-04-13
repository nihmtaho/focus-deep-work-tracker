//! Timer Display Module
//!
//! Provides a digital flip-clock style timer display component for rendering
//! both Pomodoro and freeform session timers.
//!
//! Two display modes:
//! - Freeform: HH:MM:SS or HHH:MM:SS format (via `render_for_width`)
//! - Pomodoro: MM:SS format (via `render_for_width_pomodoro`) — digits render
//!   ~60% larger because pomodoro sessions always fit in two-digit minutes.

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

    /// Render as HH:MM:SS or HHH:MM:SS (for freeform / dashboard timers).
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

    /// Render as MM:SS for Pomodoro display.
    /// Pomodoro sessions are always < 100 minutes in practice, so this compact
    /// format lets digits render considerably larger for the same terminal width.
    pub fn render_pomodoro(&self) -> String {
        let total_secs = self.duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Check if this duration requires HHH:MM:SS format (>= 100 hours)
    pub fn requires_extended_format(&self) -> bool {
        self.duration.as_secs() >= 360000
    }

    /// Render the timer as 5 rows of THICK big digits using filled block characters.
    /// Each digit is 5 chars wide × 5 rows tall.  Colon is 3 chars wide × 5 rows.
    /// Returns exactly 5 strings (one per row).
    /// Needs ~43 chars of horizontal space for "HH:MM:SS".
    pub fn render_big(&self) -> Vec<String> {
        Self::render_big_str(&self.render())
    }

    /// Render the timer as 5 rows of thin box-drawing digits.
    /// Each digit is 3 chars wide × 5 rows tall. Needs ~31 chars for "HH:MM:SS".
    pub fn render_big_thin(&self) -> Vec<String> {
        Self::render_big_thin_str(&self.render())
    }

    /// Choose thick or thin big rendering based on available width (HH:MM:SS mode).
    /// width >= 48 → thick (5-wide digits), width >= 34 → thin (3-wide), else plain text.
    pub fn render_for_width(&self, width: u16) -> (Vec<String>, bool) {
        if width >= 48 {
            (self.render_big(), true)
        } else if width >= 34 {
            (self.render_big_thin(), true)
        } else {
            (vec![self.render()], false)
        }
    }

    /// Choose rendering for Pomodoro (MM:SS) based on available width.
    ///
    /// MM:SS has 5 chars (4 digits + 1 colon):
    ///   thick needs ~27 chars  → threshold 30
    ///   thin  needs ~19 chars  → threshold 22
    ///   else plain "MM:SS" text
    pub fn render_for_width_pomodoro(&self, width: u16) -> (Vec<String>, bool) {
        let s = self.render_pomodoro();
        if width >= 30 {
            (Self::render_big_str(&s), true)
        } else if width >= 22 {
            (Self::render_big_thin_str(&s), true)
        } else {
            (vec![s], false)
        }
    }

    // ── Internal rendering helpers ─────────────────────────────────────────────

    fn render_big_str(s: &str) -> Vec<String> {
        let mut rows: Vec<String> = vec![String::new(); 5];
        for (i, ch) in s.chars().enumerate() {
            let part = Self::char_to_big(ch);
            for row in 0..5 {
                if i > 0 {
                    rows[row].push(' ');
                }
                rows[row].push_str(part[row]);
            }
        }
        rows
    }

    fn render_big_thin_str(s: &str) -> Vec<String> {
        let mut rows: Vec<String> = vec![String::new(); 5];
        for (i, ch) in s.chars().enumerate() {
            let part = Self::char_to_big_thin(ch);
            for row in 0..5 {
                if i > 0 {
                    rows[row].push(' ');
                }
                rows[row].push_str(part[row]);
            }
        }
        rows
    }

    // ── Thick digits (5 × 5 using █) ─────────────────────────────────────────
    //
    // Digit shapes follow the tock (nwtnni/tock) 5×3 bitmap layout, scaled to
    // 5-wide to fill the character grid more naturally.

    fn char_to_big(ch: char) -> [&'static str; 5] {
        match ch {
            //  ███
            // █   █
            // █   █
            // █   █
            //  ███
            '0' => [" ███ ", "█   █", "█   █", "█   █", " ███ "],
            //   █
            //  ██
            //   █
            //   █
            // █████
            '1' => ["  █  ", " ██  ", "  █  ", "  █  ", "█████"],
            // ████
            //    ██
            //  ███
            // ██
            // █████
            '2' => ["████ ", "   ██", " ███ ", "██   ", "█████"],
            //  ████
            //     █
            //  ████
            //     █
            //  ████
            '3' => [" ████", "    █", " ████", "    █", " ████"],
            // █   █
            // █   █
            // █████
            //     █
            //     █
            '4' => ["█   █", "█   █", "█████", "    █", "    █"],
            // █████
            // █
            // ████
            //     █
            // ████
            '5' => ["█████", "█    ", "████ ", "    █", "████ "],
            //  ███
            // █
            // ████
            // █   █
            //  ███
            '6' => [" ███ ", "█    ", "████ ", "█   █", " ███ "],
            // █████
            //    ██
            //   ██
            //  ██
            //  ██
            '7' => ["█████", "   ██", "  ██ ", " ██  ", " ██  "],
            //  ███
            // █   █
            //  ███
            // █   █
            //  ███
            '8' => [" ███ ", "█   █", " ███ ", "█   █", " ███ "],
            //  ███
            // █   █
            //  ████
            //     █
            //  ████
            '9' => [" ███ ", "█   █", " ████", "    █", " ████"],
            ':' => ["   ", " █ ", "   ", " █ ", "   "],
            _ =>   ["     ", "     ", "     ", "     ", "     "],
        }
    }

    // ── Thin digits (3 × 5 using box-drawing) ────────────────────────────────

    fn char_to_big_thin(ch: char) -> [&'static str; 5] {
        match ch {
            '0' => ["┌─┐", "│ │", "│ │", "│ │", "└─┘"],
            '1' => [" ╷ ", " │ ", " │ ", " │ ", " ╵ "],
            '2' => ["╶─┐", "  │", "┌─┘", "│  ", "└─╴"],
            '3' => ["╶─┐", "  │", "╶─┤", "  │", "╶─┘"],
            '4' => ["╷ ╷", "│ │", "└─┤", "  │", "  ╵"],
            '5' => ["┌─╴", "│  ", "└─┐", "  │", "╶─┘"],
            '6' => ["┌─╴", "│  ", "├─┐", "│ │", "└─┘"],
            '7' => ["╶─┐", "  │", "  │", "  │", "  ╵"],
            '8' => ["┌─┐", "│ │", "├─┤", "│ │", "└─┘"],
            '9' => ["┌─┐", "│ │", "└─┤", "  │", "╶─┘"],
            // Round dots instead of ╷·╵ for better cross-terminal rendering
            ':' => ["   ", " ● ", "   ", " ● ", "   "],
            _ =>   ["   ", "   ", "   ", "   ", "   "],
        }
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

    #[test]
    fn test_render_pomodoro_formats_mm_ss() {
        let display = TimerDisplay::new(Duration::from_secs(25 * 60)); // 25:00
        assert_eq!(display.render_pomodoro(), "25:00");

        let display = TimerDisplay::new(Duration::from_secs(0));
        assert_eq!(display.render_pomodoro(), "00:00");

        let display = TimerDisplay::new(Duration::from_secs(5 * 60 + 37)); // 5:37
        assert_eq!(display.render_pomodoro(), "05:37");
    }

    #[test]
    fn test_render_for_width_pomodoro_big_at_30() {
        let display = TimerDisplay::new(Duration::from_secs(25 * 60));
        let (rows, is_big) = display.render_for_width_pomodoro(30);
        assert!(is_big);
        assert_eq!(rows.len(), 5);
    }

    #[test]
    fn test_render_for_width_pomodoro_thin_at_22() {
        let display = TimerDisplay::new(Duration::from_secs(25 * 60));
        let (rows, is_big) = display.render_for_width_pomodoro(22);
        assert!(is_big);
        assert_eq!(rows.len(), 5);
    }

    #[test]
    fn test_render_for_width_pomodoro_plain_below_22() {
        let display = TimerDisplay::new(Duration::from_secs(25 * 60));
        let (rows, is_big) = display.render_for_width_pomodoro(21);
        assert!(!is_big);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], "25:00");
    }
}
