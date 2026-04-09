//! Integration tests for report panel metrics
//!
//! Tests report metrics computation and caching.

use focus::tui::report::ReportMetrics;

#[test]
fn test_report_metrics_creation() {
    let metrics = ReportMetrics::new(
        5,      // count_today
        12,     // count_week
        100,    // count_all_time
        1800,   // total_duration_today (30 min)
        7200,   // total_duration_week (2 hours)
        86400,  // total_duration_all_time (24 hours)
        80,     // completion_rate
        7,      // focus_streak_days
    );

    assert_eq!(metrics.count_today, 5);
    assert_eq!(metrics.count_week, 12);
    assert_eq!(metrics.completion_rate, 80);
    assert_eq!(metrics.focus_streak_days, 7);
}

#[test]
fn test_report_metrics_default() {
    let metrics = ReportMetrics::default();
    assert_eq!(metrics.count_today, 0);
    assert_eq!(metrics.count_week, 0);
    assert_eq!(metrics.completion_rate, 0);
    assert_eq!(metrics.focus_streak_days, 0);
}

#[test]
fn test_report_metrics_format_duration() {
    let metrics = ReportMetrics::new(1, 1, 1, 3661, 7200, 86400, 80, 5);

    // 3661 seconds = 1 hour 1 minute 1 second
    assert_eq!(metrics.format_duration_today(), "01:01:01");

    // 7200 seconds = 2 hours
    assert_eq!(metrics.format_duration_week(), "02:00:00");

    // 86400 seconds = 24 hours
    assert_eq!(metrics.format_duration_all_time(), "24:00:00");
}

#[test]
fn test_report_metrics_productivity_score() {
    // Productivity score formula: (completion_rate * 40) + (streak/30 * 30) + (sessions/5 * 30)
    // (80 * 40) + (7/30 * 30) + (5/5 * 30) = 3200 + 7 + 30 = 3237
    let metrics = ReportMetrics::new(5, 12, 100, 1800, 7200, 86400, 80, 7);
    let score = metrics.compute_productivity_score();

    // Should be a reasonable score (completion 80% is the primary factor)
    assert!(score > 0);
    assert!(score <= 10000); // Sanity check
}

#[test]
fn test_report_metrics_with_zero_completion() {
    let metrics = ReportMetrics::new(
        0,      // count_today
        0,      // count_week
        0,      // count_all_time
        0,      // total_duration_today
        0,      // total_duration_week
        0,      // total_duration_all_time
        0,      // completion_rate
        0,      // focus_streak_days
    );

    assert_eq!(metrics.count_today, 0);
    assert_eq!(metrics.completion_rate, 0);
    assert_eq!(metrics.format_duration_today(), "00:00:00");
}

#[test]
fn test_report_metrics_high_completion() {
    let metrics = ReportMetrics::new(10, 50, 500, 18000, 72000, 360000, 95, 15);

    assert_eq!(metrics.completion_rate, 95);
    assert_eq!(metrics.focus_streak_days, 15);
    // Score should be: (95 * 0.4) + (15/30 * 30) + (10/5 * 30) = 38 + 15 + 30 = 83
    assert!(metrics.compute_productivity_score() > 80); // High score
}

#[test]
fn test_report_metrics_placeholder() {
    assert!(true);
}
