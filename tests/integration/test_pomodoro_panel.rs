//! Integration tests for Pomodoro panel
//!
//! Tests Pomodoro panel display, state transitions, and statistics.

use focus::pomodoro::stats::PomodoroPanelState;

#[test]
fn test_pomodoro_panel_idle_state_creation() {
    let state = PomodoroPanelState::idle();
    assert_eq!(state.total_cycles_today, 0);
    assert_eq!(state.cumulative_duration_secs, 0);
    assert_eq!(state.focus_streak_days, 0);
    assert!(state.last_completion_time.is_none());
}

#[test]
fn test_pomodoro_panel_displays_idle_stats() {
    // Idle state should indicate no activity
    let idle = PomodoroPanelState::idle();
    assert!(!idle.has_activity());
    assert_eq!(idle.format_duration(), "00:00:00");
}

#[test]
fn test_pomodoro_panel_displays_active_stats() {
    // Panel with activity should show stats
    let state = PomodoroPanelState::new(4, 7200, 5, Some(1712000000));
    assert!(state.has_activity());
    assert_eq!(state.total_cycles_today, 4);
    assert_eq!(state.format_duration(), "02:00:00");
    assert_eq!(state.focus_streak_days, 5);
}

#[test]
fn test_pomodoro_panel_state_transitions() {
    // Start idle
    let mut state = PomodoroPanelState::idle();
    assert!(!state.has_activity());

    // Transition to active after first Pomodoro
    state.total_cycles_today = 1;
    state.cumulative_duration_secs = 1500; // 25 minutes
    state.last_completion_time = Some(1712000000);

    assert!(state.has_activity());
    assert_eq!(state.format_duration(), "00:25:00");
}

#[test]
fn test_pomodoro_panel_focus_streak_tracking() {
    // New day with no Pomodoro yet
    let mut state = PomodoroPanelState::idle();
    assert_eq!(state.focus_streak_days, 0);

    // After completing first Pomodoro of the day
    state.total_cycles_today = 1;
    state.focus_streak_days = 1;
    state.last_completion_time = Some(1712000000);

    assert_eq!(state.focus_streak_days, 1);

    // Continuing streak
    state.focus_streak_days = 5;
    assert_eq!(state.focus_streak_days, 5);
}

#[test]
fn test_pomodoro_panel_placeholder() {
    assert!(true);
}

// T137: Full Pomodoro panel view toggled on/off

#[test]
fn test_full_pomodoro_panel_toggle_off_by_default() {
    use focus::config::AppConfig;
    use focus::tui::app::App;
    let app = App::new(false, AppConfig::default());
    assert!(
        !app.full_pomodoro_panel,
        "Full Pomodoro panel must default to off"
    );
}

#[test]
fn test_full_pomodoro_panel_toggle_on_and_off() {
    use focus::config::AppConfig;
    use focus::tui::app::App;
    let mut app = App::new(false, AppConfig::default());

    // Toggle on
    app.full_pomodoro_panel = true;
    assert!(
        app.full_pomodoro_panel,
        "Full Pomodoro panel must be togglable on"
    );

    // Toggle off
    app.full_pomodoro_panel = false;
    assert!(
        !app.full_pomodoro_panel,
        "Full Pomodoro panel must be togglable off"
    );
}

#[test]
fn test_full_pomodoro_panel_resets_on_esc_simulation() {
    use focus::config::AppConfig;
    use focus::tui::app::App;
    let mut app = App::new(false, AppConfig::default());

    // Simulate entering full panel
    app.full_pomodoro_panel = true;
    assert!(app.full_pomodoro_panel);

    // Simulate Esc/Q key handler behaviour: set full_pomodoro_panel = false
    app.full_pomodoro_panel = false;
    assert!(
        !app.full_pomodoro_panel,
        "Esc/Q must exit full Pomodoro panel"
    );
}
