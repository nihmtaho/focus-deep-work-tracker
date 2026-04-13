//! Pomodoro statistics and panel state tracking
//!
//! Provides historical statistics about Pomodoro sessions for dashboard display.

/// Statistics about Pomodoro sessions for panel display
#[derive(Debug, Clone)]
pub struct PomodoroPanelState {
    /// Total completed work cycles today
    pub total_cycles_today: u32,
    /// Total duration of all work cycles today (in seconds)
    pub cumulative_duration_secs: u64,
    /// Current focus streak (consecutive days with at least one Pomodoro)
    pub focus_streak_days: u32,
    /// Timestamp of last completed work phase (None if none today)
    pub last_completion_time: Option<i64>,
}

impl PomodoroPanelState {
    /// Create an idle panel state (no sessions today)
    pub fn idle() -> Self {
        Self {
            total_cycles_today: 0,
            cumulative_duration_secs: 0,
            focus_streak_days: 0,
            last_completion_time: None,
        }
    }

    /// Create panel state from current stats
    pub fn new(
        total_cycles_today: u32,
        cumulative_duration_secs: u64,
        focus_streak_days: u32,
        last_completion_time: Option<i64>,
    ) -> Self {
        Self {
            total_cycles_today,
            cumulative_duration_secs,
            focus_streak_days,
            last_completion_time,
        }
    }

    /// Format cumulative duration as HH:MM:SS
    pub fn format_duration(&self) -> String {
        let hours = self.cumulative_duration_secs / 3600;
        let minutes = (self.cumulative_duration_secs % 3600) / 60;
        let seconds = self.cumulative_duration_secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Check if there have been any Pomodoros today
    pub fn has_activity(&self) -> bool {
        self.total_cycles_today > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_pomodoro_panel_idle_state() {
        let state = PomodoroPanelState::idle();
        assert_eq!(state.total_cycles_today, 0);
        assert_eq!(state.cumulative_duration_secs, 0);
        assert_eq!(state.focus_streak_days, 0);
        assert!(state.last_completion_time.is_none());
        assert!(!state.has_activity());
    }

    #[test]
    fn test_pomodoro_panel_with_stats() {
        let state = PomodoroPanelState::new(3, 2700, 5, Some(1712000000));
        assert_eq!(state.total_cycles_today, 3);
        assert_eq!(state.cumulative_duration_secs, 2700); // 45 minutes
        assert_eq!(state.focus_streak_days, 5);
        assert!(state.last_completion_time.is_some());
        assert!(state.has_activity());
    }

    #[test]
    fn test_pomodoro_panel_format_duration() {
        // Test 45 minutes (2700 seconds)
        let state = PomodoroPanelState::new(3, 2700, 5, None);
        assert_eq!(state.format_duration(), "00:45:00");

        // Test 1 hour 30 minutes (5400 seconds)
        let state = PomodoroPanelState::new(3, 5400, 5, None);
        assert_eq!(state.format_duration(), "01:30:00");

        // Test 2 hours 15 minutes 30 seconds (8130 seconds)
        let state = PomodoroPanelState::new(3, 8130, 5, None);
        assert_eq!(state.format_duration(), "02:15:30");

        // Test 0 duration
        let state = PomodoroPanelState::idle();
        assert_eq!(state.format_duration(), "00:00:00");
    }

    #[test]
    fn test_pomodoro_panel_focus_streak() {
        let state = PomodoroPanelState::new(2, 1800, 7, Some(Utc::now().timestamp()));
        assert_eq!(state.focus_streak_days, 7);

        let idle = PomodoroPanelState::idle();
        assert_eq!(idle.focus_streak_days, 0);
    }
}
