//! Integration tests for keyboard binding system
//!
//! Tests keyboard event handling, context switching, and shortcut resolution.

use focus::tui::keyboard::{KeyHandler, KeyContext, KeyAction, TabTarget};
use crossterm::event::{KeyCode, KeyEvent};

fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, crossterm::event::KeyModifiers::empty())
}

#[test]
fn test_keyboard_handler_basic_creation() {
    let handler = KeyHandler::new(false);
    assert_eq!(handler.get_context(), KeyContext::Viewing);
}

#[test]
fn test_context_switching() {
    let mut handler = KeyHandler::new(false);
    handler.set_context(KeyContext::Input);
    assert_eq!(handler.get_context(), KeyContext::Input);
    handler.set_context(KeyContext::Viewing);
    assert_eq!(handler.get_context(), KeyContext::Viewing);
}

#[test]
fn test_letter_shortcuts_in_viewing_mode() {
    let handler = KeyHandler::new(false);

    let d_event = create_key_event(KeyCode::Char('d'));
    assert_eq!(handler.handle_key(d_event), KeyAction::NavigateTab(TabTarget::Dashboard));

    let s_event = create_key_event(KeyCode::Char('s'));
    assert_eq!(handler.handle_key(s_event), KeyAction::NavigateTab(TabTarget::Sessions));

    let t_event = create_key_event(KeyCode::Char('t'));
    assert_eq!(handler.handle_key(t_event), KeyAction::NavigateTab(TabTarget::TODOs));
}

#[test]
fn test_esc_cancels_input() {
    let mut handler = KeyHandler::new(false);
    handler.set_context(KeyContext::Input);
    let esc_event = create_key_event(KeyCode::Esc);
    assert_eq!(handler.handle_key(esc_event), KeyAction::CancelInput);
}
