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

    /// Choose the best thick or thin big rendering based on available width
    /// (HH:MM:SS mode).  Selects from 6 size tiers (13-, 11-, 9-, 7-, 5-wide
    /// thick and 3-wide thin) down to plain text for very narrow panels.
    pub fn render_for_width(&self, width: u16) -> (Vec<String>, bool) {
        let s = self.render();
        match Self::best_digit_width_hms(width) {
            None => (vec![s], false),
            Some(dw) => (Self::render_big_sized(&s, dw), true),
        }
    }

    /// Choose the best rendering for Pomodoro (MM:SS) based on available width.
    /// Selects from 6 size tiers down to plain text for very narrow panels.
    pub fn render_for_width_pomodoro(&self, width: u16) -> (Vec<String>, bool) {
        let s = self.render_pomodoro();
        match Self::best_digit_width_pomo(width) {
            None => (vec![s], false),
            Some(dw) => (Self::render_big_sized(&s, dw), true),
        }
    }

    // ── Width-tier selection ──────────────────────────────────────────────────

    /// Return the best thick-digit width for the available panel area (HH:MM:SS
    /// / HHH:MM:SS strings, 8–9 chars).
    ///
    /// Total width for HH:MM:SS = 6×w + 2×(w−2) + 7 spaces = 8w + 3.
    ///
    /// | digit width | HH:MM:SS total | threshold |
    /// |-------------|---------------|-----------|
    /// | 29          | 235 cols       | 240       |
    /// | 25          | 203 cols       | 208       |
    /// | 21          | 171 cols       | 176       |
    /// | 17          | 139 cols       | 144       |
    /// | 13          | 107 cols       | 112       |
    /// | 11          |  91 cols       |  96       |
    /// |  9          |  75 cols       |  80       |
    /// |  7          |  59 cols       |  64       |
    /// |  5 (thick)  |  43 cols       |  48       |
    /// |  3 (thin)   |  27 cols       |  34       |
    /// | plain text  |  —            | < 34      |
    ///
    /// Returns `None` for plain text, `Some(3)` for thin box-drawing,
    /// `Some(5..=29)` for the appropriate thick size.
    pub fn best_digit_width_hms(available_width: u16) -> Option<u8> {
        if available_width >= 240 {
            Some(29)
        } else if available_width >= 208 {
            Some(25)
        } else if available_width >= 176 {
            Some(21)
        } else if available_width >= 144 {
            Some(17)
        } else if available_width >= 112 {
            Some(13)
        } else if available_width >= 96 {
            Some(11)
        } else if available_width >= 80 {
            Some(9)
        } else if available_width >= 64 {
            Some(7)
        } else if available_width >= 48 {
            Some(5)
        } else if available_width >= 34 {
            Some(3) // thin box-drawing
        } else {
            None // plain text
        }
    }

    /// Return the best thick-digit width for the available panel area (MM:SS
    /// Pomodoro strings, 5 chars).
    ///
    /// Total width for MM:SS = 4×w + 1×(w−2) + 4 spaces = 5w + 2.
    ///
    /// | digit width | MM:SS total | threshold |
    /// |-------------|------------|-----------|
    /// | 45          | 227 cols    | 232       |
    /// | 41          | 207 cols    | 212       |
    /// | 37          | 187 cols    | 192       |
    /// | 33          | 167 cols    | 172       |
    /// | 29          | 147 cols    | 152       |
    /// | 25          | 127 cols    | 132       |
    /// | 21          | 107 cols    | 112       |
    /// | 17          |  87 cols    |  92       |
    /// | 13          |  67 cols    |  72       |
    /// | 11          |  57 cols    |  62       |
    /// |  9          |  47 cols    |  52       |
    /// |  7          |  37 cols    |  42       |
    /// |  5 (thick)  |  27 cols    |  30       |
    /// |  3 (thin)   |  17 cols    |  22       |
    /// | plain text  |  —         | < 22      |
    pub fn best_digit_width_pomo(available_width: u16) -> Option<u8> {
        if available_width >= 232 {
            Some(45)
        } else if available_width >= 212 {
            Some(41)
        } else if available_width >= 192 {
            Some(37)
        } else if available_width >= 172 {
            Some(33)
        } else if available_width >= 152 {
            Some(29)
        } else if available_width >= 132 {
            Some(25)
        } else if available_width >= 112 {
            Some(21)
        } else if available_width >= 92 {
            Some(17)
        } else if available_width >= 72 {
            Some(13)
        } else if available_width >= 62 {
            Some(11)
        } else if available_width >= 52 {
            Some(9)
        } else if available_width >= 42 {
            Some(7)
        } else if available_width >= 30 {
            Some(5)
        } else if available_width >= 22 {
            Some(3) // thin box-drawing
        } else {
            None // plain text
        }
    }

    /// Render `s` using thick digits of size `digit_width × digit_height_for_width(digit_width)`.
    /// Pass `3` to get the thin box-drawing renderer instead.
    pub fn render_big_sized(s: &str, digit_width: u8) -> Vec<String> {
        if digit_width == 3 {
            return Self::render_big_thin_str(s);
        }
        let w = digit_width as usize;
        let h = Self::digit_height_for_width(digit_width) as usize;
        let mut rows: Vec<String> = vec![String::new(); h];
        for (i, ch) in s.chars().enumerate() {
            if i > 0 {
                for row in &mut rows {
                    row.push(' ');
                }
            }
            let part = Self::make_digit_rows(ch, w, h);
            for (r, row) in rows.iter_mut().enumerate() {
                row.push_str(&part[r]);
            }
        }
        rows
    }

    // ── Internal rendering helpers ─────────────────────────────────────────────

    /// Render `curr` with per-character opacity-fade animation at `digit_width`.
    ///
    /// Digit height is derived from `digit_height_for_width(digit_width)`, so both
    /// dimensions scale together (5×5, 7×7, 9×9, 11×11, 13×13).
    /// Pass `3` for thin box-drawing (no animation).
    ///
    /// Animation is 6 frames / 300 ms:
    ///   frames 1–3 fade the *old* digit out, frames 4–6 fade the *new* digit in.
    pub fn render_animated_big_sized(
        curr: &str,
        prev: &str,
        anim_frames: &[u8],
        digit_width: u8,
    ) -> Vec<String> {
        if digit_width == 3 {
            return Self::render_big_thin_str(curr);
        }
        let w = digit_width as usize;
        let h = Self::digit_height_for_width(digit_width) as usize;
        let mut rows: Vec<String> = vec![String::new(); h];
        let prev_chars: Vec<char> = prev.chars().collect();

        for (i, ch) in curr.chars().enumerate() {
            if i > 0 {
                for row in &mut rows {
                    row.push(' ');
                }
            }

            let frame = anim_frames.get(i).copied().unwrap_or(0);
            let prev_ch = prev_chars.get(i).copied().unwrap_or(ch);

            if frame == 0 || prev_ch == ch {
                let part = Self::make_digit_rows(ch, w, h);
                for (r, row) in rows.iter_mut().enumerate() {
                    row.push_str(&part[r]);
                }
            } else {
                let curr_part = Self::make_digit_rows(ch, w, h);
                let prev_part = Self::make_digit_rows(prev_ch, w, h);
                for r in 0..h {
                    rows[r].push_str(&Self::blend_digit_row(&curr_part[r], &prev_part[r], frame));
                }
            }
        }

        rows
    }

    /// Convenience wrapper — animated rendering at the default 5-wide thick size.
    /// Delegates to [`render_animated_big_sized`] with `digit_width = 5`.
    pub fn render_animated_big_str(curr: &str, prev: &str, anim_frames: &[u8]) -> Vec<String> {
        Self::render_animated_big_sized(curr, prev, anim_frames, 5)
    }

    /// Blend two 5-char wide digit rows using shade chars.
    ///
    /// `frame` 1–3: fading OUT `prev_row` (█ → ▓ → ▒ → gone).
    /// `frame` 4–6: fading IN  `curr_row` (░ → ▒ → ▓ → gone / next frame = done).
    fn blend_digit_row(curr_row: &str, prev_row: &str, frame: u8) -> String {
        let curr_chars: Vec<char> = curr_row.chars().collect();
        let prev_chars: Vec<char> = prev_row.chars().collect();
        let len = curr_chars.len().max(prev_chars.len());

        let mut out = String::with_capacity(len * 3); // multi-byte chars
        for i in 0..len {
            let cc = curr_chars.get(i).copied().unwrap_or(' ');
            let pc = prev_chars.get(i).copied().unwrap_or(' ');

            let ch = if frame <= 3 {
                // Fading out old character
                if pc == ' ' {
                    ' '
                } else {
                    match frame {
                        1 => '█',
                        2 => '▓',
                        _ => '▒', // frame 3
                    }
                }
            } else {
                // Fading in new character
                if cc == ' ' {
                    ' '
                } else {
                    match frame {
                        4 => '░',
                        5 => '▒',
                        _ => '▓', // frame 6
                    }
                }
            };
            out.push(ch);
        }
        out
    }

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

    pub fn render_big_thin_str(s: &str) -> Vec<String> {
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
            // █████
            // ██ ██
            // ██ ██
            // ██ ██
            // █████
            '0' => ["█████", "██ ██", "██ ██", "██ ██", "█████"],
            //   ██
            //  ███
            //   ██
            //   ██
            //   ██
            '1' => ["  ██ ", " ███ ", "  ██ ", "  ██ ", "  ██ "],
            // █████
            //    ██
            // █████
            // ██
            // █████
            '2' => ["█████", "   ██", "█████", "██   ", "█████"],
            // █████
            //    ██
            // █████
            //    ██
            // █████
            '3' => ["█████", "   ██", "█████", "   ██", "█████"],
            // ██ ██
            // ██ ██
            // █████
            //    ██
            //    ██
            '4' => ["██ ██", "██ ██", "█████", "   ██", "   ██"],
            // █████
            // ██
            // █████
            //    ██
            // █████
            '5' => ["█████", "██   ", "█████", "   ██", "█████"],
            // █████
            // ██
            // █████
            // ██ ██
            // █████
            '6' => ["█████", "██   ", "█████", "██ ██", "█████"],
            // █████
            //    ██
            //    ██
            //    ██
            //    ██
            '7' => ["█████", "   ██", "   ██", "   ██", "   ██"],
            // █████
            // ██ ██
            // █████
            // ██ ██
            // █████
            '8' => ["█████", "██ ██", "█████", "██ ██", "█████"],
            // █████
            // ██ ██
            // █████
            //    ██
            // █████
            '9' => ["█████", "██ ██", "█████", "   ██", "█████"],
            ':' => ["   ", " █ ", "   ", " █ ", "   "],
            _ => ["     ", "     ", "     ", "     ", "     "],
        }
    }

    // ── Responsive digit sizing ───────────────────────────────────────────────

    /// Map digit width to digit height.  All thick sizes ≥ 5 use a square
    /// (w×w) grid so digits scale proportionally at every tier.
    fn digit_height_for_width(w: u8) -> u8 {
        if w >= 5 {
            w
        } else {
            5
        }
    }

    /// Generate `height` rows of block-character pixels for a single character
    /// at the given `width`.  All digit sizes share the same 7-segment geometry;
    /// both the horizontal bar thickness and the internal gap scale with the
    /// digit dimensions so large sizes remain legible.
    ///
    /// - **bar_h**: horizontal bar thickness = max(1, height/10)
    /// - **inner_gap**: internal horizontal gap (always odd) = max(1, width/9)|1
    /// - **stroke**: vertical stroke width = (width − inner_gap) / 2
    ///
    /// The character set is: `'0'`–`'9'`, `':'`, and a blank fallback.
    fn make_digit_rows(ch: char, width: usize, height: usize) -> Vec<String> {
        // Horizontal bar thickness — 1 row up to h=9, then scales (1 per 10 rows)
        let bar_h = (height / 10).max(1);
        // Internal horizontal gap — forced odd so left/right strokes are equal width
        let gap_raw = (width / 9).max(1);
        let inner_gap = if gap_raw.is_multiple_of(2) {
            gap_raw + 1
        } else {
            gap_raw
        };
        // Vertical stroke width
        let stroke = (width - inner_gap) / 2;

        // Upper / lower section heights (rows of open sides between horizontal bars)
        let inner_h = height.saturating_sub(3 * bar_h);
        let upper = inner_h / 2;
        let lower = inner_h - upper; // may be 1 more than upper when inner_h is odd

        // Row building blocks (all exactly `width` chars wide)
        let full = "█".repeat(width);
        let blank = " ".repeat(width);
        let sides = format!(
            "{}{}{}",
            "█".repeat(stroke),
            " ".repeat(inner_gap),
            "█".repeat(stroke)
        );
        let right = format!("{}{}", " ".repeat(stroke + inner_gap), "█".repeat(stroke));
        let left = format!("{}{}", "█".repeat(stroke), " ".repeat(stroke + inner_gap));

        // Colon: full-width rows with a single centered dot
        let cdot = format!(
            "{}█{}",
            " ".repeat(width / 2),
            " ".repeat(width - width / 2 - 1)
        );
        let cblk = " ".repeat(width);

        let mut rows: Vec<String> = Vec::with_capacity(height);

        match ch {
            // 0: open box (top bar + sides + bottom bar)
            '0' => {
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..(height - 2 * bar_h) {
                    rows.push(sides.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
            }
            // 1: right-side vertical bar
            '1' => {
                for _ in 0..height {
                    rows.push(right.clone());
                }
            }
            // 2: top-right, middle, bottom-left
            '2' => {
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..upper {
                    rows.push(right.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..lower {
                    rows.push(left.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
            }
            // 3: top-right, middle, bottom-right
            '3' => {
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..upper {
                    rows.push(right.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..lower {
                    rows.push(right.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
            }
            // 4: top-sides, middle bar, bottom-right (no outer top/bottom bar)
            '4' => {
                for _ in 0..(upper + bar_h) {
                    rows.push(sides.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..(lower + bar_h) {
                    rows.push(right.clone());
                }
            }
            // 5: top-left, middle, bottom-right
            '5' => {
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..upper {
                    rows.push(left.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..lower {
                    rows.push(right.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
            }
            // 6: top-left, middle, bottom-sides
            '6' => {
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..upper {
                    rows.push(left.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..lower {
                    rows.push(sides.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
            }
            // 7: top bar then right-side descent
            '7' => {
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in bar_h..height {
                    rows.push(right.clone());
                }
            }
            // 8: full box with middle bar
            '8' => {
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..upper {
                    rows.push(sides.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..lower {
                    rows.push(sides.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
            }
            // 9: top-sides, middle, bottom-right
            '9' => {
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..upper {
                    rows.push(sides.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
                for _ in 0..lower {
                    rows.push(right.clone());
                }
                for _ in 0..bar_h {
                    rows.push(full.clone());
                }
            }
            // ':': two centered dots at ⅓ and ⅔ height
            ':' => {
                for row in 0..height {
                    if row == height / 3 || row == 2 * height / 3 {
                        rows.push(cdot.clone());
                    } else {
                        rows.push(cblk.clone());
                    }
                }
            }
            _ => {
                for _ in 0..height {
                    rows.push(blank.clone());
                }
            }
        }

        debug_assert_eq!(rows.len(), height, "make_digit_rows: row count mismatch");
        rows
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
            _ => ["   ", "   ", "   ", "   ", "   "],
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
