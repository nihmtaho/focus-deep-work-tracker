//! Report panel module
//!
//! Provides report panel functionality for displaying session analytics and metrics.
//! Will include daily/weekly trends, completion rates, focus streaks, and productivity scores.

/// Report metrics for display
#[derive(Debug, Clone)]
pub struct ReportMetrics {
    pub count_today: i64,
    pub count_week: i64,
    pub count_all_time: i64,
    pub total_duration_today: u64,
    pub completion_rate: f64,
    pub focus_streak: u32,
}

impl ReportMetrics {
    /// Create new metrics with default values
    pub fn new() -> Self {
        Self {
            count_today: 0,
            count_week: 0,
            count_all_time: 0,
            total_duration_today: 0,
            completion_rate: 0.0,
            focus_streak: 0,
        }
    }
}

impl Default for ReportMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_metrics() {
        let metrics = ReportMetrics::new();
        assert_eq!(metrics.count_today, 0);
        assert_eq!(metrics.completion_rate, 0.0);
    }
}
