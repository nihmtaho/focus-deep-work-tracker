//! Integration tests for timer display component
//!
//! Tests the flip-clock timer display rendering in various formats.

use focus::tui::timer_display::TimerDisplay;
use std::time::Duration;

#[test]
fn test_flip_clock_format_hh_mm_ss_for_under_100_hours() {
    let display = TimerDisplay::new(Duration::from_secs(3661)); // 1:01:01
    assert_eq!(display.render(), "01:01:01");
}

#[test]
fn test_flip_clock_format_hhh_mm_ss_for_over_100_hours() {
    let display = TimerDisplay::new(Duration::from_secs(360000)); // 100:00:00
    assert_eq!(display.render(), "100:00:00");
}

#[test]
fn test_timer_display_zero_seconds() {
    let display = TimerDisplay::new(Duration::from_secs(0));
    assert_eq!(display.render(), "00:00:00");
}

#[test]
fn test_timer_display_boundary_99_59_59() {
    let display = TimerDisplay::new(Duration::from_secs(359999)); // 99:59:59
    assert_eq!(display.render(), "99:59:59");
}

#[test]
fn test_timer_display_various_durations() {
    let test_cases = vec![
        (0, "00:00:00"),
        (1, "00:00:01"),
        (60, "00:01:00"),
        (3600, "01:00:00"),
        (3661, "01:01:01"),
        (359999, "99:59:59"),
        (360000, "100:00:00"),
    ];

    for (secs, expected) in test_cases {
        let display = TimerDisplay::new(Duration::from_secs(secs));
        assert_eq!(
            display.render(),
            expected,
            "Timer display failed for {} seconds",
            secs
        );
    }
}
