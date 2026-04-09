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

#[test]
fn test_letter_shortcuts_navigation_dashboard() {
    let handler = KeyHandler::new(false);
    let d_event = create_key_event(KeyCode::Char('d'));
    let action = handler.handle_key(d_event);
    assert_eq!(action, KeyAction::NavigateTab(TabTarget::Dashboard));
}

#[test]
fn test_letter_shortcuts_navigation_sessions() {
    let handler = KeyHandler::new(false);
    let s_event = create_key_event(KeyCode::Char('s'));
    let action = handler.handle_key(s_event);
    assert_eq!(action, KeyAction::NavigateTab(TabTarget::Sessions));
}

#[test]
fn test_letter_shortcuts_navigation_todos() {
    let handler = KeyHandler::new(false);
    let t_event = create_key_event(KeyCode::Char('t'));
    let action = handler.handle_key(t_event);
    assert_eq!(action, KeyAction::NavigateTab(TabTarget::TODOs));
}

#[test]
fn test_letter_shortcuts_rapid_navigation() {
    let handler = KeyHandler::new(false);

    // Simulate rapid key presses: d -> s -> t -> d
    let actions = vec![
        handler.handle_key(create_key_event(KeyCode::Char('d'))),
        handler.handle_key(create_key_event(KeyCode::Char('s'))),
        handler.handle_key(create_key_event(KeyCode::Char('t'))),
        handler.handle_key(create_key_event(KeyCode::Char('d'))),
    ];

    let expected = vec![
        KeyAction::NavigateTab(TabTarget::Dashboard),
        KeyAction::NavigateTab(TabTarget::Sessions),
        KeyAction::NavigateTab(TabTarget::TODOs),
        KeyAction::NavigateTab(TabTarget::Dashboard),
    ];

    assert_eq!(actions, expected);
}

#[test]
fn test_letter_shortcuts_disabled_in_input_mode() {
    let mut handler = KeyHandler::new(false);
    handler.set_context(KeyContext::Input);

    // In input mode, letter shortcuts should not work
    let d_event = create_key_event(KeyCode::Char('d'));
    let action = handler.handle_key(d_event);

    // Should return InputKeypress, not NavigateTab
    match action {
        KeyAction::InputKeypress(_) => {} // Expected
        _ => panic!("Letter shortcuts should be disabled in Input mode"),
    }
}

#[test]
fn test_letter_shortcuts_vim_mode_navigation_still_works() {
    let handler = KeyHandler::new(true); // vim_mode enabled

    // Letter shortcuts like 'd' should still navigate even with vim mode enabled
    // because they are context shortcuts, not movement keys
    let d_event = create_key_event(KeyCode::Char('d'));
    assert_eq!(handler.handle_key(d_event), KeyAction::NavigateTab(TabTarget::Dashboard));

    // Vim mode hjkl should still work
    let h_event = create_key_event(KeyCode::Char('h'));
    assert_eq!(handler.handle_key(h_event), KeyAction::FocusLeft);
}
