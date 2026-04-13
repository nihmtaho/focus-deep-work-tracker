/// Reusable vim-aware text input component.
///
/// Encapsulates buffer, cursor position, and vim Normal/Insert mode.
/// Used by all text entry fields in the TUI (todo add, session name, tag, rename).
use crossterm::event::KeyCode;

// ── Mode ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum VimInputMode {
    Normal,
    Insert,
}

// ── Result ─────────────────────────────────────────────────────────────────────

pub enum TextInputEvent {
    /// Keep editing — no action required.
    Continue,
    /// User confirmed — contains the trimmed buffer text.
    Submit(String),
    /// User cancelled (Esc in Normal mode or Esc in non-vim mode).
    Cancel,
}

// ── TextInput ──────────────────────────────────────────────────────────────────

pub struct TextInput {
    pub buffer: String,
    pub cursor_pos: usize,
    pub vim_mode: VimInputMode,
    /// True when the app-wide vim mode setting is enabled.
    pub vim_enabled: bool,
}

impl TextInput {
    pub fn new(vim_enabled: bool) -> Self {
        Self {
            buffer: String::new(),
            cursor_pos: 0,
            vim_mode: VimInputMode::Insert,
            vim_enabled,
        }
    }

    /// Reset to empty, ready for fresh input. Starts in Insert mode.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.cursor_pos = 0;
        self.vim_mode = VimInputMode::Insert;
    }

    /// Pre-fill with an existing value (e.g. rename prompts). Cursor goes to end.
    pub fn set_value(&mut self, s: &str) {
        self.buffer = s.to_string();
        self.cursor_pos = s.len();
        self.vim_mode = VimInputMode::Insert;
    }

    /// Sync the vim_enabled flag (call when the setting changes at runtime).
    pub fn set_vim_enabled(&mut self, enabled: bool) {
        self.vim_enabled = enabled;
    }

    /// Process a key event and return what the caller should do.
    pub fn handle_key(&mut self, key: KeyCode) -> TextInputEvent {
        if self.vim_enabled {
            match self.vim_mode {
                VimInputMode::Insert => self.handle_insert(key),
                VimInputMode::Normal => self.handle_normal(key),
            }
        } else {
            self.handle_simple(key)
        }
    }

    /// Return the buffer content trimmed.
    pub fn value(&self) -> String {
        self.buffer.trim().to_string()
    }

    /// Returns `(before_cursor, cursor_char, after_cursor)` strings for rendering.
    /// `cursor_char` is the char at cursor (space if past end).
    pub fn cursor_spans(&self) -> (&str, String, &str) {
        let pos = self.cursor_pos.min(self.buffer.len());
        let before = &self.buffer[..pos];
        let (cursor_char, after_start) = if pos < self.buffer.len() {
            let ch = self.buffer[pos..].chars().next().unwrap();
            (ch.to_string(), pos + ch.len_utf8())
        } else {
            (" ".to_string(), pos)
        };
        let after = &self.buffer[after_start..];
        (before, cursor_char, after)
    }

    /// Mode label suitable for display (`-- INSERT --`, `-- NORMAL --`, or `""`).
    pub fn mode_label(&self) -> &'static str {
        if !self.vim_enabled {
            return "";
        }
        match self.vim_mode {
            VimInputMode::Insert => "-- INSERT --",
            VimInputMode::Normal => "-- NORMAL --",
        }
    }

    /// Hint text for the current mode.
    pub fn hint(&self) -> &'static str {
        if !self.vim_enabled {
            return "[Enter] save  [Esc] cancel";
        }
        match self.vim_mode {
            VimInputMode::Insert => "[Esc] normal  [Enter] save",
            VimInputMode::Normal => {
                "[i/a/A/I] insert  [h/l] move  [w/b] word  [x] del  [D] del→end  [Enter] save  [Esc] cancel"
            }
        }
    }

    // ── Internal key handlers ──────────────────────────────────────────────────

    fn handle_simple(&mut self, key: KeyCode) -> TextInputEvent {
        match key {
            KeyCode::Enter => TextInputEvent::Submit(self.value()),
            KeyCode::Esc => TextInputEvent::Cancel,
            KeyCode::Char(c) => {
                self.insert_char(c);
                TextInputEvent::Continue
            }
            KeyCode::Backspace => {
                self.backspace();
                TextInputEvent::Continue
            }
            KeyCode::Delete => {
                self.delete_at_cursor();
                TextInputEvent::Continue
            }
            KeyCode::Left => {
                self.move_left();
                TextInputEvent::Continue
            }
            KeyCode::Right => {
                self.move_right();
                TextInputEvent::Continue
            }
            _ => TextInputEvent::Continue,
        }
    }

    fn handle_insert(&mut self, key: KeyCode) -> TextInputEvent {
        match key {
            KeyCode::Esc => {
                self.vim_mode = VimInputMode::Normal;
                self.clamp_normal();
                TextInputEvent::Continue
            }
            KeyCode::Enter => TextInputEvent::Submit(self.value()),
            KeyCode::Char(c) => {
                self.insert_char(c);
                TextInputEvent::Continue
            }
            KeyCode::Backspace => {
                self.backspace();
                TextInputEvent::Continue
            }
            KeyCode::Delete => {
                self.delete_at_cursor();
                TextInputEvent::Continue
            }
            KeyCode::Left => {
                self.move_left();
                TextInputEvent::Continue
            }
            KeyCode::Right => {
                self.move_right();
                TextInputEvent::Continue
            }
            _ => TextInputEvent::Continue,
        }
    }

    fn handle_normal(&mut self, key: KeyCode) -> TextInputEvent {
        match key {
            KeyCode::Char('i') => {
                self.vim_mode = VimInputMode::Insert;
                TextInputEvent::Continue
            }
            KeyCode::Char('a') => {
                self.move_right();
                self.vim_mode = VimInputMode::Insert;
                TextInputEvent::Continue
            }
            KeyCode::Char('A') => {
                self.cursor_pos = self.buffer.len();
                self.vim_mode = VimInputMode::Insert;
                TextInputEvent::Continue
            }
            KeyCode::Char('I') => {
                self.cursor_pos = 0;
                self.vim_mode = VimInputMode::Insert;
                TextInputEvent::Continue
            }
            KeyCode::Enter => TextInputEvent::Submit(self.value()),
            KeyCode::Esc => TextInputEvent::Cancel,
            KeyCode::Char('h') | KeyCode::Left => {
                self.move_left();
                TextInputEvent::Continue
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.move_right_normal();
                TextInputEvent::Continue
            }
            KeyCode::Char('0') | KeyCode::Home => {
                self.cursor_pos = 0;
                TextInputEvent::Continue
            }
            KeyCode::Char('$') | KeyCode::End => {
                self.end_normal();
                TextInputEvent::Continue
            }
            KeyCode::Char('w') => {
                self.word_forward();
                TextInputEvent::Continue
            }
            KeyCode::Char('b') => {
                self.word_backward();
                TextInputEvent::Continue
            }
            KeyCode::Char('x') => {
                self.delete_at_cursor();
                self.clamp_normal();
                TextInputEvent::Continue
            }
            KeyCode::Char('D') => {
                self.buffer.truncate(self.cursor_pos);
                self.clamp_normal();
                TextInputEvent::Continue
            }
            _ => TextInputEvent::Continue,
        }
    }

    // ── Buffer primitives ──────────────────────────────────────────────────────

    fn insert_char(&mut self, c: char) {
        if self.buffer.len() < 256 {
            self.buffer.insert(self.cursor_pos, c);
            self.cursor_pos += c.len_utf8();
        }
    }

    fn move_left(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let mut pos = self.cursor_pos - 1;
        while pos > 0 && !self.buffer.is_char_boundary(pos) {
            pos -= 1;
        }
        self.cursor_pos = pos;
    }

    fn move_right(&mut self) {
        let len = self.buffer.len();
        if self.cursor_pos >= len {
            return;
        }
        let mut pos = self.cursor_pos + 1;
        while pos <= len && !self.buffer.is_char_boundary(pos) {
            pos += 1;
        }
        self.cursor_pos = pos;
    }

    fn move_right_normal(&mut self) {
        let len = self.buffer.len();
        if self.buffer.is_empty() || self.cursor_pos >= len.saturating_sub(1) {
            return;
        }
        self.move_right();
    }

    fn clamp_normal(&mut self) {
        if self.buffer.is_empty() {
            self.cursor_pos = 0;
            return;
        }
        let last = self.buffer.char_indices().last().map(|(i, _)| i).unwrap_or(0);
        if self.cursor_pos > last {
            self.cursor_pos = last;
        }
        while self.cursor_pos > 0 && !self.buffer.is_char_boundary(self.cursor_pos) {
            self.cursor_pos -= 1;
        }
    }

    fn backspace(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        self.move_left();
        self.buffer.remove(self.cursor_pos);
    }

    fn delete_at_cursor(&mut self) {
        let pos = self.cursor_pos;
        if pos >= self.buffer.len() {
            return;
        }
        self.buffer.remove(pos);
        if self.cursor_pos > self.buffer.len() {
            self.cursor_pos = self.buffer.len();
        }
    }

    fn end_normal(&mut self) {
        if self.buffer.is_empty() {
            self.cursor_pos = 0;
            return;
        }
        self.cursor_pos = self.buffer.char_indices().last().map(|(i, _)| i).unwrap_or(0);
    }

    fn word_forward(&mut self) {
        let chars: Vec<(usize, char)> = self.buffer.char_indices().collect();
        let start = chars.iter().position(|&(i, _)| i == self.cursor_pos).unwrap_or(0);
        let mut i = start;
        while i < chars.len() && !chars[i].1.is_whitespace() {
            i += 1;
        }
        while i < chars.len() && chars[i].1.is_whitespace() {
            i += 1;
        }
        self.cursor_pos = chars.get(i).map(|&(j, _)| j).unwrap_or(self.buffer.len());
        self.clamp_normal();
    }

    fn word_backward(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let chars: Vec<(usize, char)> = self.buffer.char_indices().collect();
        let idx = chars
            .iter()
            .position(|&(i, _)| i >= self.cursor_pos)
            .unwrap_or(chars.len())
            .saturating_sub(1);
        let mut i = idx;
        while i > 0 && chars[i].1.is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].1.is_whitespace() {
            i -= 1;
        }
        self.cursor_pos = chars.get(i).map(|&(j, _)| j).unwrap_or(0);
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn input(vim: bool) -> TextInput {
        TextInput::new(vim)
    }

    #[test]
    fn simple_type_and_submit() {
        let mut t = input(false);
        assert!(matches!(t.handle_key(KeyCode::Char('h')), TextInputEvent::Continue));
        assert!(matches!(t.handle_key(KeyCode::Char('i')), TextInputEvent::Continue));
        assert_eq!(t.buffer, "hi");
        assert!(matches!(t.handle_key(KeyCode::Enter), TextInputEvent::Submit(s) if s == "hi"));
    }

    #[test]
    fn simple_esc_cancels() {
        let mut t = input(false);
        t.handle_key(KeyCode::Char('x'));
        assert!(matches!(t.handle_key(KeyCode::Esc), TextInputEvent::Cancel));
    }

    #[test]
    fn simple_backspace() {
        let mut t = input(false);
        t.handle_key(KeyCode::Char('a'));
        t.handle_key(KeyCode::Char('b'));
        t.handle_key(KeyCode::Backspace);
        assert_eq!(t.buffer, "a");
        assert_eq!(t.cursor_pos, 1);
    }

    #[test]
    fn vim_starts_in_insert() {
        let t = input(true);
        assert_eq!(t.vim_mode, VimInputMode::Insert);
    }

    #[test]
    fn vim_esc_switches_to_normal() {
        let mut t = input(true);
        t.handle_key(KeyCode::Char('a'));
        t.handle_key(KeyCode::Esc);
        assert_eq!(t.vim_mode, VimInputMode::Normal);
    }

    #[test]
    fn vim_normal_i_returns_to_insert() {
        let mut t = input(true);
        t.handle_key(KeyCode::Esc); // → Normal (buffer empty, stays at 0)
        t.handle_key(KeyCode::Char('i'));
        assert_eq!(t.vim_mode, VimInputMode::Insert);
    }

    #[test]
    fn vim_normal_esc_cancels() {
        let mut t = input(true);
        t.handle_key(KeyCode::Char('a'));
        t.handle_key(KeyCode::Esc); // → Normal
        assert!(matches!(t.handle_key(KeyCode::Esc), TextInputEvent::Cancel));
    }

    #[test]
    fn vim_x_deletes_char_at_cursor() {
        let mut t = input(true);
        t.set_value("hello");
        t.cursor_pos = 0;
        t.vim_mode = VimInputMode::Normal;
        t.handle_key(KeyCode::Char('x'));
        assert_eq!(t.buffer, "ello");
    }

    #[test]
    fn vim_d_deletes_to_end() {
        let mut t = input(true);
        t.set_value("hello world");
        t.cursor_pos = 5;
        t.vim_mode = VimInputMode::Normal;
        t.handle_key(KeyCode::Char('D'));
        assert_eq!(t.buffer, "hello");
    }

    #[test]
    fn cursor_spans_mid_buffer() {
        let mut t = input(false);
        t.set_value("abc");
        t.cursor_pos = 1;
        let (before, cur, after) = t.cursor_spans();
        assert_eq!(before, "a");
        assert_eq!(cur, "b");
        assert_eq!(after, "c");
    }

    #[test]
    fn cursor_spans_end_of_buffer() {
        let mut t = input(false);
        t.set_value("ab");
        t.cursor_pos = 2;
        let (before, cur, after) = t.cursor_spans();
        assert_eq!(before, "ab");
        assert_eq!(cur, " ");
        assert_eq!(after, "");
    }
}
