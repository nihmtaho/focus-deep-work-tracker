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
}

impl KeyHandler {
    /// Create a new KeyHandler with the specified vim_mode setting
    pub fn new(vim_mode: bool) -> Self {
        Self {
            vim_mode,
            context: KeyContext::Viewing,
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

    /// Handle a keyboard event and return the resulting action
    pub fn handle_key(&self, key: crossterm::event::KeyEvent) -> KeyAction {
        use crossterm::event::KeyCode;

        // Always handle Esc in input mode first
        if self.context == KeyContext::Input {
            if key.code == KeyCode::Esc {
                return KeyAction::CancelInput;
            }
            // In input mode, allow all other keys for text entry
            return KeyAction::InputKeypress(key);
        }

        // In viewing mode, handle navigation shortcuts
        match key.code {
            // Tab navigation (letter shortcuts)
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
        let handler = KeyHandler::new(false);
        let event = create_key_event(KeyCode::Char('d'));
        assert_eq!(handler.handle_key(event), KeyAction::NavigateTab(TabTarget::Dashboard));
    }

    #[test]
    fn test_letter_shortcut_s_navigates_to_sessions() {
        let handler = KeyHandler::new(false);
        let event = create_key_event(KeyCode::Char('s'));
        assert_eq!(handler.handle_key(event), KeyAction::NavigateTab(TabTarget::Sessions));
    }

    #[test]
    fn test_letter_shortcut_t_navigates_to_todos() {
        let handler = KeyHandler::new(false);
        let event = create_key_event(KeyCode::Char('t'));
        assert_eq!(handler.handle_key(event), KeyAction::NavigateTab(TabTarget::TODOs));
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
            KeyAction::InputKeypress(_) => {},
            _ => panic!("Expected InputKeypress in input mode"),
        }
    }

    #[test]
    fn test_vim_mode_hjkl_navigation() {
        let handler = KeyHandler::new(true);

        assert_eq!(handler.handle_key(create_key_event(KeyCode::Char('h'))), KeyAction::FocusLeft);
        assert_eq!(handler.handle_key(create_key_event(KeyCode::Char('j'))), KeyAction::FocusDown);
        assert_eq!(handler.handle_key(create_key_event(KeyCode::Char('k'))), KeyAction::FocusUp);
        assert_eq!(handler.handle_key(create_key_event(KeyCode::Char('l'))), KeyAction::FocusRight);
    }

    #[test]
    fn test_arrow_keys_when_vim_mode_disabled() {
        let handler = KeyHandler::new(false);

        assert_eq!(handler.handle_key(create_key_event(KeyCode::Left)), KeyAction::FocusLeft);
        assert_eq!(handler.handle_key(create_key_event(KeyCode::Right)), KeyAction::FocusRight);
        assert_eq!(handler.handle_key(create_key_event(KeyCode::Up)), KeyAction::FocusUp);
        assert_eq!(handler.handle_key(create_key_event(KeyCode::Down)), KeyAction::FocusDown);
    }
}
