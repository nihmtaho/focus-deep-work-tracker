//! Integration tests for keyboard binding system
//!
//! Tests keyboard event handling, context switching, and shortcut resolution.

use crossterm::event::{KeyCode, KeyEvent};
use focus::tui::keyboard::{KeyAction, KeyContext, KeyHandler, TabTarget};

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
    let mut handler = KeyHandler::new(false);

    let d_event = create_key_event(KeyCode::Char('d'));
    assert_eq!(
        handler.handle_key(d_event),
        KeyAction::NavigateTab(TabTarget::Dashboard)
    );

    let s_event = create_key_event(KeyCode::Char('s'));
    assert_eq!(
        handler.handle_key(s_event),
        KeyAction::NavigateTab(TabTarget::Sessions)
    );

    let t_event = create_key_event(KeyCode::Char('t'));
    assert_eq!(
        handler.handle_key(t_event),
        KeyAction::NavigateTab(TabTarget::TODOs)
    );
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
    let mut handler = KeyHandler::new(false);
    let d_event = create_key_event(KeyCode::Char('d'));
    let action = handler.handle_key(d_event);
    assert_eq!(action, KeyAction::NavigateTab(TabTarget::Dashboard));
}

#[test]
fn test_letter_shortcuts_navigation_sessions() {
    let mut handler = KeyHandler::new(false);
    let s_event = create_key_event(KeyCode::Char('s'));
    let action = handler.handle_key(s_event);
    assert_eq!(action, KeyAction::NavigateTab(TabTarget::Sessions));
}

#[test]
fn test_letter_shortcuts_navigation_todos() {
    let mut handler = KeyHandler::new(false);
    let t_event = create_key_event(KeyCode::Char('t'));
    let action = handler.handle_key(t_event);
    assert_eq!(action, KeyAction::NavigateTab(TabTarget::TODOs));
}

#[test]
fn test_letter_shortcuts_rapid_navigation() {
    let mut handler = KeyHandler::new(false);

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
    let mut handler = KeyHandler::new(true); // vim_mode enabled

    // In vim mode, 'd' starts dd command composition — does NOT navigate to Dashboard.
    // This is per spec FR-016: single 'd' in vim mode must not trigger tab navigation.
    let d_event = create_key_event(KeyCode::Char('d'));
    assert_eq!(
        handler.handle_key(d_event),
        KeyAction::None // waiting for second 'd'
    );

    // Vim mode hjkl should still work
    let h_event = create_key_event(KeyCode::Char('h'));
    assert_eq!(handler.handle_key(h_event), KeyAction::FocusLeft);
}

// T106: Panel focus indicator integration tests

#[test]
fn test_number_shortcuts_focus_panels() {
    let mut handler = KeyHandler::new(false);

    // 1/2/3 map to FocusPanel(0/1/2) in viewing mode
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
fn test_panel_focus_out_of_range_ignored() {
    let mut handler = KeyHandler::new(false);

    // 4+ returns None — out-of-range panel indices are not focused
    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Char('4'))),
        KeyAction::None
    );
    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Char('9'))),
        KeyAction::None
    );
}

#[test]
fn test_panel_focus_disabled_in_input_mode() {
    let mut handler = KeyHandler::new(false);
    handler.set_context(KeyContext::Input);

    // In input mode, number keys are treated as text input, not panel focus
    let one_event = create_key_event(KeyCode::Char('1'));
    match handler.handle_key(one_event) {
        KeyAction::InputKeypress(_) => {} // expected
        other => panic!("Expected InputKeypress in input mode, got {:?}", other),
    }
}

// T113: vim keys hjkl map correctly when vim_mode=true

#[test]
fn test_vim_keys_hjkl_map_when_vim_mode_enabled() {
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

// T114: vim keys ignored when vim_mode=false

#[test]
fn test_vim_keys_ignored_when_vim_mode_disabled() {
    let mut handler = KeyHandler::new(false);

    // hjkl should not produce navigation actions when vim_mode=false
    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Char('h'))),
        KeyAction::None
    );
    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Char('j'))),
        KeyAction::None
    );
    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Char('k'))),
        KeyAction::None
    );
    // Note: 'l' does not produce a navigation action (same as 'h'/'j'/'k')
    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Char('l'))),
        KeyAction::None
    );
}

// T115: hjkl navigation works when vim_mode=true, arrow keys work when vim_mode=false

#[test]
fn test_vim_mode_hjkl_navigation_in_viewing_context() {
    let mut handler = KeyHandler::new(true);

    // All four directions should work via hjkl
    let left = handler.handle_key(create_key_event(KeyCode::Char('h')));
    let down = handler.handle_key(create_key_event(KeyCode::Char('j')));
    let up = handler.handle_key(create_key_event(KeyCode::Char('k')));
    let right = handler.handle_key(create_key_event(KeyCode::Char('l')));

    assert_eq!(left, KeyAction::FocusLeft);
    assert_eq!(down, KeyAction::FocusDown);
    assert_eq!(up, KeyAction::FocusUp);
    assert_eq!(right, KeyAction::FocusRight);
}

#[test]
fn test_arrow_keys_navigation_when_vim_mode_disabled() {
    let mut handler = KeyHandler::new(false);

    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Left)),
        KeyAction::FocusLeft
    );
    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Down)),
        KeyAction::FocusDown
    );
    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Up)),
        KeyAction::FocusUp
    );
    assert_eq!(
        handler.handle_key(create_key_event(KeyCode::Right)),
        KeyAction::FocusRight
    );
}

#[test]
fn test_vim_keys_disabled_in_input_context() {
    let mut handler = KeyHandler::new(true);
    handler.set_context(KeyContext::Input);

    // hjkl should return InputKeypress in Input context, not navigation
    for key in [
        KeyCode::Char('h'),
        KeyCode::Char('j'),
        KeyCode::Char('k'),
        KeyCode::Char('l'),
    ] {
        match handler.handle_key(create_key_event(key)) {
            KeyAction::InputKeypress(_) => {}
            other => panic!(
                "Expected InputKeypress for {:?} in input context, got {:?}",
                key, other
            ),
        }
    }
}

// ── US1: Delete Key Conflict Resolution ─────────────────────────────────────

#[test]
fn test_delete_key_emits_delete_item_action() {
    let mut handler = KeyHandler::new(false);
    let event = create_key_event(KeyCode::Delete);
    assert_eq!(handler.handle_key(event), KeyAction::DeleteItem);
}

#[test]
fn test_backspace_key_emits_delete_item_action() {
    let mut handler = KeyHandler::new(false);
    let event = create_key_event(KeyCode::Backspace);
    assert_eq!(handler.handle_key(event), KeyAction::DeleteItem);
}

#[test]
fn test_d_key_always_navigates_to_dashboard_in_normal_mode() {
    // d must navigate regardless of context; no delete action
    let mut handler = KeyHandler::new(false);
    let event = create_key_event(KeyCode::Char('d'));
    assert_eq!(
        handler.handle_key(event),
        KeyAction::NavigateTab(TabTarget::Dashboard)
    );
}

#[test]
fn test_delete_key_in_input_context_is_input_keypress() {
    // In Input context, Backspace edits the text field, not deletes the todo
    let mut handler = KeyHandler::new(false);
    handler.set_context(KeyContext::Input);
    let event = create_key_event(KeyCode::Backspace);
    match handler.handle_key(event) {
        KeyAction::InputKeypress(_) => {}
        other => panic!("Expected InputKeypress in Input context, got {:?}", other),
    }
}

// ── US7: Full Vim Mode Navigation ────────────────────────────────────────────

#[test]
fn test_vim_big_g_emits_jump_bottom() {
    let mut handler = KeyHandler::new(true);
    let event = create_key_event(KeyCode::Char('G'));
    assert_eq!(handler.handle_key(event), KeyAction::JumpBottom);
}

#[test]
fn test_vim_big_g_ignored_when_vim_mode_disabled() {
    let mut handler = KeyHandler::new(false);
    let event = create_key_event(KeyCode::Char('G'));
    // In normal mode G has no binding
    assert_eq!(handler.handle_key(event), KeyAction::None);
}

#[test]
fn test_vim_dd_within_timeout_emits_delete() {
    let mut handler = KeyHandler::new(true);
    // First d: sets pending, returns None
    let first = handler.handle_key(create_key_event(KeyCode::Char('d')));
    assert_eq!(first, KeyAction::None, "first d should return None");
    // Second d immediately: returns DeleteItem
    let second = handler.handle_key(create_key_event(KeyCode::Char('d')));
    assert_eq!(
        second,
        KeyAction::DeleteItem,
        "second d should return DeleteItem"
    );
}

#[test]
fn test_vim_d_single_does_not_navigate_in_vim_mode() {
    let mut handler = KeyHandler::new(true);
    let event = create_key_event(KeyCode::Char('d'));
    // In vim mode, single d waits for command — must NOT navigate to Dashboard
    let action = handler.handle_key(event);
    assert_ne!(
        action,
        KeyAction::NavigateTab(TabTarget::Dashboard),
        "single d in vim mode must not navigate to Dashboard"
    );
}

#[test]
fn test_vim_gg_within_timeout_emits_jump_top() {
    let mut handler = KeyHandler::new(true);
    // First g: sets pending, returns None
    let first = handler.handle_key(create_key_event(KeyCode::Char('g')));
    assert_eq!(first, KeyAction::None, "first g should return None");
    // Second g immediately: returns JumpTop
    let second = handler.handle_key(create_key_event(KeyCode::Char('g')));
    assert_eq!(second, KeyAction::JumpTop, "second g should return JumpTop");
}

#[test]
fn test_vim_g_ignored_when_vim_mode_disabled() {
    let mut handler = KeyHandler::new(false);
    let event = create_key_event(KeyCode::Char('g'));
    assert_eq!(handler.handle_key(event), KeyAction::None);
}
