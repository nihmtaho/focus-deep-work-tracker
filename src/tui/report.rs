//! Report panel module
//!
//! Provides report panel functionality for displaying session analytics and metrics.
//! Includes daily/weekly trends, completion rates, focus streaks, and productivity scores.

/// Report metrics for display on the Report panel
#[derive(Debug, Clone)]
pub struct ReportMetrics {
    /// Number of sessions completed today
    pub count_today: u32,
    /// Number of sessions completed this week
    pub count_week: u32,
    /// Total sessions completed (all time)
    pub count_all_time: u32,
    /// Total duration of sessions today (seconds)
    pub total_duration_today: u64,
    /// Total duration of sessions this week (seconds)
    pub total_duration_week: u64,
    /// Total duration of sessions all time (seconds)
    pub total_duration_all_time: u64,
    /// Percentage of TODOs completed today
    pub completion_rate: u32,
    /// Number of consecutive days with at least one session
    pub focus_streak_days: u32,
}

impl ReportMetrics {
    /// Create new metrics with specified values
    pub fn new(
        count_today: u32,
        count_week: u32,
        count_all_time: u32,
        total_duration_today: u64,
        total_duration_week: u64,
        total_duration_all_time: u64,
        completion_rate: u32,
        focus_streak_days: u32,
    ) -> Self {
        Self {
            count_today,
            count_week,
            count_all_time,
            total_duration_today,
            total_duration_week,
            total_duration_all_time,
            completion_rate,
            focus_streak_days,
        }
    }

    /// Format today's duration as HH:MM:SS
    pub fn format_duration_today(&self) -> String {
        Self::format_duration(self.total_duration_today)
    }

    /// Format this week's duration as HH:MM:SS
    pub fn format_duration_week(&self) -> String {
        Self::format_duration(self.total_duration_week)
    }

    /// Format all-time duration as HH:MM:SS
    pub fn format_duration_all_time(&self) -> String {
        Self::format_duration(self.total_duration_all_time)
    }

    /// Format duration in seconds to HH:MM:SS format
    fn format_duration(secs: u64) -> String {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Compute productivity score based on metrics
    /// Formula: (completion_rate * 40) + (streak/30 * 30) + (sessions/5 * 30)
    /// Max score: 100 (when completion=100%, streak=30+, sessions=5+)
    pub fn compute_productivity_score(&self) -> u32 {
        let completion_score = (self.completion_rate as f64) * 0.4;
        let streak_score = ((self.focus_streak_days as f64 / 30.0) * 30.0).min(30.0);
        let session_score = ((self.count_today as f64 / 5.0) * 30.0).min(30.0);

        (completion_score + streak_score + session_score) as u32
    }
}

impl Default for ReportMetrics {
    fn default() -> Self {
        Self::new(0, 0, 0, 0, 0, 0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_metrics() {
        let metrics = ReportMetrics::new(0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(metrics.count_today, 0);
        assert_eq!(metrics.completion_rate, 0);
    }

    #[test]
    fn test_format_duration() {
        let metrics = ReportMetrics::new(1, 1, 1, 3661, 7200, 86400, 80, 5);
        assert_eq!(metrics.format_duration_today(), "01:01:01");
        assert_eq!(metrics.format_duration_week(), "02:00:00");
        assert_eq!(metrics.format_duration_all_time(), "24:00:00");
    }

    #[test]
    fn test_productivity_score() {
        let metrics = ReportMetrics::new(5, 12, 100, 1800, 7200, 86400, 80, 7);
        let score = metrics.compute_productivity_score();
        assert!(score > 0);
        assert!(score <= 100);
    }
}
