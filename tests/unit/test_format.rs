/// Unit tests for src/display/format.rs
///
/// Constitution Principle II: tests verify format_duration contract
/// defined in contracts/cli-schema.md "Elapsed Time Format" section.
use chrono::Duration;
use focus::display::format::format_duration;

// Contract: seconds only when duration < 1 minute
#[test]
fn test_format_seconds_only() {
    assert_eq!(format_duration(Duration::seconds(45)), "45s");
    assert_eq!(format_duration(Duration::seconds(1)), "1s");
    assert_eq!(format_duration(Duration::seconds(59)), "59s");
}

// Contract: minutes and seconds when duration >= 1 minute but < 1 hour
#[test]
fn test_format_minutes_and_seconds() {
    assert_eq!(format_duration(Duration::seconds(192)), "3m 12s");
    assert_eq!(format_duration(Duration::seconds(60)), "1m 0s");
    assert_eq!(format_duration(Duration::seconds(3599)), "59m 59s");
}

// Contract: hours, minutes, and seconds when duration >= 1 hour
#[test]
fn test_format_hours_minutes_seconds() {
    assert_eq!(format_duration(Duration::seconds(5025)), "1h 23m 45s");
    assert_eq!(format_duration(Duration::seconds(3600)), "1h 0m 0s");
    assert_eq!(format_duration(Duration::seconds(7322)), "2h 2m 2s");
}

// Edge: zero duration
#[test]
fn test_format_zero_duration() {
    assert_eq!(format_duration(Duration::seconds(0)), "0s");
}

// Edge: negative duration clamps to 0s (max(0) applied)
#[test]
fn test_format_negative_duration_clamps_to_zero() {
    assert_eq!(format_duration(Duration::seconds(-10)), "0s");
}
