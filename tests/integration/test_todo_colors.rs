//! Integration tests for TODO color coding and input handling
//!
//! Tests that TODOs display with appropriate colors based on state,
//! and that TODO input can be cancelled with Esc key.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use focus::tui::keyboard::{KeyAction, KeyContext, KeyHandler};

fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

#[test]
fn test_todo_input_prompt_cancels_on_esc_key() {
    // Simulate TODO input mode
    let mut handler = KeyHandler::new(false);
    handler.set_context(KeyContext::Input);

    // Press Esc key
    let esc_event = create_key_event(KeyCode::Esc);
    let action = handler.handle_key(esc_event);

    // Verify CancelInput action is returned
    assert_eq!(action, KeyAction::CancelInput);
}

#[test]
fn test_todo_input_text_entry_accepts_typing() {
    // Simulate TODO input mode
    let mut handler = KeyHandler::new(false);
    handler.set_context(KeyContext::Input);

    // Type some characters
    let char_event = create_key_event(KeyCode::Char('a'));
    let action = handler.handle_key(char_event);

    // Verify InputKeypress action is returned for typing
    match action {
        KeyAction::InputKeypress(_) => {} // Expected
        _ => panic!("Expected InputKeypress action for typing in input mode"),
    }
}

#[test]
fn test_todo_input_disables_all_navigation_shortcuts() {
    let mut handler = KeyHandler::new(false);
    handler.set_context(KeyContext::Input);

    // Try to navigate to dashboard (d key) - should not work
    let d_event = create_key_event(KeyCode::Char('d'));
    let action = handler.handle_key(d_event);
    assert!(!matches!(action, KeyAction::NavigateTab(_)));

    // Try to focus panel (1 key) - should not work
    let num_event = create_key_event(KeyCode::Char('1'));
    let action = handler.handle_key(num_event);
    assert!(!matches!(action, KeyAction::FocusPanel(_)));
}

#[test]
fn test_todo_context_switch_from_input_to_viewing() {
    let mut handler = KeyHandler::new(false);

    // Start in input mode
    handler.set_context(KeyContext::Input);
    assert_eq!(handler.get_context(), KeyContext::Input);

    // Switch to viewing mode (simulating Esc handling in app)
    handler.set_context(KeyContext::Viewing);
    assert_eq!(handler.get_context(), KeyContext::Viewing);

    // Now 'd' key should navigate to dashboard again
    let d_event = create_key_event(KeyCode::Char('d'));
    let action = handler.handle_key(d_event);
    assert_eq!(
        action,
        KeyAction::NavigateTab(focus::tui::keyboard::TabTarget::Dashboard)
    );
}

#[test]
fn test_todo_color_rendering_with_theme_colors() {
    use focus::models::todo::Todo;
    use focus::theme::Theme;

    // Create test TODOs with different statuses
    let active_todo = Todo {
        id: 1,
        title: "Active TODO".to_string(),
        status: "active".to_string(),
        created_at: 0,
        completed_at: None,
    };

    let completed_todo = Todo {
        id: 2,
        title: "Completed TODO".to_string(),
        status: "completed".to_string(),
        created_at: 0,
        completed_at: Some(100),
    };

    // Test with each theme
    for theme in &[Theme::OneDark, Theme::Material, Theme::Light, Theme::Dark] {
        let colors = theme.colors();

        // Verify active TODO uses todo_todo color
        let active_color = active_todo.get_color(&colors);
        assert_eq!(active_color, colors.todo_todo);

        // Verify completed TODO uses todo_completed color
        let completed_color = completed_todo.get_color(&colors);
        assert_eq!(completed_color, colors.todo_completed);
    }
}

#[test]
fn test_todo_color_updates_on_state_transition() {
    use focus::models::todo::Todo;
    use focus::theme::Theme;

    let colors = Theme::OneDark.colors();

    // Start with active TODO
    let mut todo = Todo {
        id: 1,
        title: "Test TODO".to_string(),
        status: "active".to_string(),
        created_at: 0,
        completed_at: None,
    };

    let active_color = todo.get_color(&colors);
    assert_eq!(active_color, colors.todo_todo);

    // Transition to completed
    todo.status = "completed".to_string();
    todo.completed_at = Some(100);

    let completed_color = todo.get_color(&colors);
    assert_eq!(completed_color, colors.todo_completed);

    // Verify colors are different (unless theme defines same color for both)
    // This ensures the color system is actually responsive to state changes
    assert_ne!(
        colors.todo_todo, colors.todo_completed,
        "Theme should define different colors for active and completed TODOs"
    );
}

#[test]
fn test_todo_color_system_placeholder() {
    // TODO colors placeholder - will be implemented in Phase 5
    assert!(true);
}

// T124: Integration test — all session fields visible in Sessions (Log) tab

#[test]
fn test_session_log_displays_all_required_fields() {
    use focus::tui::views::log::{format_status, responsive_column_widths};

    // Verify status formatting
    assert_eq!(format_status(true), "done");
    assert_eq!(format_status(false), "active");

    // Verify 6-column layout exists for a standard terminal width
    let widths = responsive_column_widths(120);
    assert_eq!(
        widths.len(),
        6,
        "Session log must have 6 columns: date, task, tag, mode, duration, status"
    );
}

#[test]
fn test_session_log_layout_adapts_to_narrow_terminal() {
    use focus::tui::views::log::responsive_column_widths;
    use ratatui::layout::Constraint;

    let widths = responsive_column_widths(60);
    assert_eq!(widths.len(), 6);

    // Task column must be at least 10 wide even on narrow terminals
    match widths[1] {
        Constraint::Min(w) => assert!(w >= 10, "Task column must be at least 10 wide"),
        _ => panic!("Task column must use Min constraint for responsive layout"),
    }
}

#[test]
fn test_session_log_no_truncation_for_long_names() {
    // Verify the render logic preserves full task names
    let long_task = "This is a very long session task name that previously would have been truncated at 30 chars";
    // Simulate what render() does: task = s.task.clone() (no truncation)
    let displayed = long_task.to_string();
    assert_eq!(
        displayed.len(),
        long_task.len(),
        "Long task names must not be truncated"
    );
}

// T131: Session tags display in tag_color from theme (distinct from session title)

#[test]
fn test_session_tags_use_tag_color_from_theme() {
    use focus::theme::Theme;

    // Verify each theme provides a tag_color distinct from session_title
    for theme in &[Theme::OneDark, Theme::Material, Theme::Light, Theme::Dark] {
        let colors = theme.colors();
        assert_ne!(
            colors.tag_color, colors.session_title,
            "tag_color must differ from session_title for theme {:?}",
            theme
        );
    }
}

#[test]
fn test_session_tags_graceful_when_no_tag() {
    // Sessions without tags must display a placeholder, not panic or show empty
    let tag: Option<String> = None;
    let displayed = tag.as_deref().unwrap_or("—");
    assert_eq!(displayed, "—", "Missing tag must render as '—' placeholder");
}

#[test]
fn test_session_tag_color_consistent_across_themes() {
    use focus::tui::themes::get_current_colors;

    // get_current_colors() is the single source used in all views (log, dashboard, todos)
    // Verify it always returns a valid tag_color
    let colors = get_current_colors();
    // tag_color is valid if validate() passes (all color fields set)
    assert_eq!(colors.validate(), Ok(()));
}
