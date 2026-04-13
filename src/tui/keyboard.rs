//! Keyboard handling module
//!
//! Manages keyboard input events with context-aware key binding resolution.
//! Supports viewing mode (navigation) and input mode (text entry) with configurable
//! vim mode bindings.

use crossterm::event::KeyEvent;

/// Keyboard context determines which keys are active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyContext {
    /// Normal viewing mode - navigation shortcuts active
    Viewing,
    /// Input mode - text entry active, shortcuts disabled
    Input,
}

/// Action triggered by keyboard input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Navigate to a specific tab
    NavigateTab(TabTarget),
    /// Focus a specific panel (0-based index)
    FocusPanel(usize),
    /// Move focus/cursor in a direction
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    /// Delete the currently selected item
    DeleteItem,
    /// Jump to the first item in the current list (vim gg)
    JumpTop,
    /// Jump to the last item in the current list (vim G)
    JumpBottom,
    /// Cancel input (Esc key)
    CancelInput,
    /// Process input keypress (typing)
    InputKeypress(KeyEvent),
    /// No action
    None,
}

/// Tab navigation targets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabTarget {
    Dashboard,
    Sessions,
    TODOs,
}

/// Keyboard handler for managing key events with context awareness
#[derive(Debug)]
pub struct KeyHandler {
    vim_mode: bool,
    context: KeyContext,
    /// Timestamp of a pending first `d` press in vim mode (for `dd` command).
    pending_d: Option<std::time::Instant>,
    /// Timestamp of a pending first `g` press in vim mode (for `gg` command).
    pending_g: Option<std::time::Instant>,
}

const VIM_COMMAND_TIMEOUT_MS: u128 = 1000;

impl KeyHandler {
    /// Create a new KeyHandler with the specified vim_mode setting
    pub fn new(vim_mode: bool) -> Self {
        Self {
            vim_mode,
            context: KeyContext::Viewing,
            pending_d: None,
            pending_g: None,
        }
    }

    /// Set the current keyboard context
    pub fn set_context(&mut self, context: KeyContext) {
        self.context = context;
    }

    /// Get the current keyboard context
    pub fn get_context(&self) -> KeyContext {
        self.context
    }

    /// Handle a keyboard event and return the resulting action.
    ///
    /// Takes `&mut self` because vim multi-key commands (`dd`, `gg`) maintain
    /// pending-command state that must be mutated between key presses.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        // Always handle Esc in input mode first
        if self.context == KeyContext::Input {
            if key.code == KeyCode::Esc {
                return KeyAction::CancelInput;
            }
            // In input mode, allow all other keys for text entry (including Backspace)
            return KeyAction::InputKeypress(key);
        }

        // In viewing mode, handle navigation shortcuts
        match key.code {
            // Deletion keys — platform independent (Delete = forward-delete / Fn+Backspace
            // on macOS; Backspace = standard backspace on all platforms)
            KeyCode::Delete | KeyCode::Backspace if self.context == KeyContext::Viewing => {
                self.pending_d = None;
                self.pending_g = None;
                KeyAction::DeleteItem
            }

            // Tab navigation (letter shortcuts)
            // In vim mode, `d` is intercepted for `dd` command composition.
            KeyCode::Char('d') if self.vim_mode && self.context == KeyContext::Viewing => {
                match self.pending_d {
                    Some(ts) if ts.elapsed().as_millis() < VIM_COMMAND_TIMEOUT_MS => {
                        // Second `d` within timeout → delete
                        self.pending_d = None;
                        KeyAction::DeleteItem
                    }
                    _ => {
                        // First `d` (or timed-out previous `d`) → start pending window
                        self.pending_d = Some(std::time::Instant::now());
                        KeyAction::None
                    }
                }
            }
            // In normal mode, `d` always navigates to Dashboard
            KeyCode::Char('d') if self.context == KeyContext::Viewing => {
                KeyAction::NavigateTab(TabTarget::Dashboard)
            }

            KeyCode::Char('s') if self.context == KeyContext::Viewing => {
                KeyAction::NavigateTab(TabTarget::Sessions)
            }
            KeyCode::Char('t') if self.context == KeyContext::Viewing => {
                KeyAction::NavigateTab(TabTarget::TODOs)
            }

            // Panel focus (number shortcuts)
            KeyCode::Char('1') if self.context == KeyContext::Viewing => KeyAction::FocusPanel(0),
            KeyCode::Char('2') if self.context == KeyContext::Viewing => KeyAction::FocusPanel(1),
            KeyCode::Char('3') if self.context == KeyContext::Viewing => KeyAction::FocusPanel(2),

            // Vim mode navigation (hjkl)
            KeyCode::Char('h') if self.vim_mode && self.context == KeyContext::Viewing => {
                KeyAction::FocusLeft
            }
            KeyCode::Char('j') if self.vim_mode && self.context == KeyContext::Viewing => {
                KeyAction::FocusDown
            }
            KeyCode::Char('k') if self.vim_mode && self.context == KeyContext::Viewing => {
                KeyAction::FocusUp
            }
            KeyCode::Char('l') if self.vim_mode && self.context == KeyContext::Viewing => {
                KeyAction::FocusRight
            }

            // Vim `G` — jump to bottom
            KeyCode::Char('G') if self.vim_mode && self.context == KeyContext::Viewing => {
                self.pending_g = None;
                KeyAction::JumpBottom
            }

            // Vim `gg` — jump to top (two-key sequence)
            KeyCode::Char('g') if self.vim_mode && self.context == KeyContext::Viewing => {
                match self.pending_g {
                    Some(ts) if ts.elapsed().as_millis() < VIM_COMMAND_TIMEOUT_MS => {
                        self.pending_g = None;
                        KeyAction::JumpTop
                    }
                    _ => {
                        self.pending_g = Some(std::time::Instant::now());
                        KeyAction::None
                    }
                }
            }

            // Arrow key navigation (when vim mode disabled)
            KeyCode::Left if !self.vim_mode && self.context == KeyContext::Viewing => {
                KeyAction::FocusLeft
            }
            KeyCode::Right if !self.vim_mode && self.context == KeyContext::Viewing => {
                KeyAction::FocusRight
            }
            KeyCode::Up if !self.vim_mode && self.context == KeyContext::Viewing => {
                KeyAction::FocusUp
            }
            KeyCode::Down if !self.vim_mode && self.context == KeyContext::Viewing => {
                KeyAction::FocusDown
            }

            _ => KeyAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::empty())
    }

    #[test]
    fn test_letter_shortcut_d_navigates_to_dashboard() {
        let mut handler = KeyHandler::new(false);
        let event = create_key_event(KeyCode::Char('d'));
        assert_eq!(
            handler.handle_key(event),
            KeyAction::NavigateTab(TabTarget::Dashboard)
        );
    }

    #[test]
    fn test_letter_shortcut_s_navigates_to_sessions() {
        let mut handler = KeyHandler::new(false);
        let event = create_key_event(KeyCode::Char('s'));
        assert_eq!(
            handler.handle_key(event),
            KeyAction::NavigateTab(TabTarget::Sessions)
        );
    }

    #[test]
    fn test_letter_shortcut_t_navigates_to_todos() {
        let mut handler = KeyHandler::new(false);
        let event = create_key_event(KeyCode::Char('t'));
        assert_eq!(
            handler.handle_key(event),
            KeyAction::NavigateTab(TabTarget::TODOs)
        );
    }

    #[test]
    fn test_esc_in_input_mode_returns_cancel() {
        let mut handler = KeyHandler::new(false);
        handler.set_context(KeyContext::Input);
        let event = create_key_event(KeyCode::Esc);
        assert_eq!(handler.handle_key(event), KeyAction::CancelInput);
    }

    #[test]
    fn test_shortcuts_disabled_in_input_mode() {
        let mut handler = KeyHandler::new(false);
        handler.set_context(KeyContext::Input);
        let event = create_key_event(KeyCode::Char('d'));
        match handler.handle_key(event) {
            KeyAction::InputKeypress(_) => {}
            _ => panic!("Expected InputKeypress in input mode"),
        }
    }

    #[test]
    fn test_vim_mode_hjkl_navigation() {
        let mut handler = KeyHandler::new(true);

        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('h'))),
            KeyAction::FocusLeft
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('j'))),
            KeyAction::FocusDown
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('k'))),
            KeyAction::FocusUp
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('l'))),
            KeyAction::FocusRight
        );
    }

    #[test]
    fn test_arrow_keys_when_vim_mode_disabled() {
        let mut handler = KeyHandler::new(false);

        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Left)),
            KeyAction::FocusLeft
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Right)),
            KeyAction::FocusRight
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Up)),
            KeyAction::FocusUp
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Down)),
            KeyAction::FocusDown
        );
    }

    #[test]
    fn test_esc_key_in_input_context_returns_cancel_input() {
        let mut handler = KeyHandler::new(false);
        handler.set_context(KeyContext::Input);
        let event = create_key_event(KeyCode::Esc);
        assert_eq!(handler.handle_key(event), KeyAction::CancelInput);
    }

    #[test]
    fn test_navigation_shortcuts_disabled_in_input_context() {
        let mut handler = KeyHandler::new(false);
        handler.set_context(KeyContext::Input);

        // Test letter shortcuts disabled
        let d_event = create_key_event(KeyCode::Char('d'));
        match handler.handle_key(d_event) {
            KeyAction::InputKeypress(_) => {}
            _ => panic!("Expected InputKeypress for 'd' in input mode"),
        }

        // Test number shortcuts disabled
        let one_event = create_key_event(KeyCode::Char('1'));
        match handler.handle_key(one_event) {
            KeyAction::InputKeypress(_) => {}
            _ => panic!("Expected InputKeypress for '1' in input mode"),
        }

        // Test arrow keys disabled
        let up_event = create_key_event(KeyCode::Up);
        match handler.handle_key(up_event) {
            KeyAction::InputKeypress(_) => {}
            _ => panic!("Expected InputKeypress for Up in input mode"),
        }
    }

    #[test]
    fn test_number_shortcuts_map_to_focus_panel() {
        let mut handler = KeyHandler::new(false);
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('1'))),
            KeyAction::FocusPanel(0)
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('2'))),
            KeyAction::FocusPanel(1)
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('3'))),
            KeyAction::FocusPanel(2)
        );
    }

    #[test]
    fn test_out_of_range_number_ignored() {
        let mut handler = KeyHandler::new(false);
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('4'))),
            KeyAction::None
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('5'))),
            KeyAction::None
        );
        assert_eq!(
            handler.handle_key(create_key_event(KeyCode::Char('9'))),
            KeyAction::None
        );
    }

    #[test]
    fn test_all_shortcuts_return_input_keypress_in_input_context() {
        let mut handler = KeyHandler::new(true); // vim mode enabled
        handler.set_context(KeyContext::Input);

        // All keys except Esc should return InputKeypress
        let test_keys = vec![
            KeyCode::Char('d'),
            KeyCode::Char('s'),
            KeyCode::Char('t'),
            KeyCode::Char('1'),
            KeyCode::Char('h'),
            KeyCode::Char('j'),
            KeyCode::Char('k'),
            KeyCode::Char('l'),
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
        ];

        for key_code in test_keys {
            let event = create_key_event(key_code);
            assert!(
                matches!(handler.handle_key(event), KeyAction::InputKeypress(_)),
                "Key {:?} should return InputKeypress in input context",
                key_code
            );
        }
    }
}
