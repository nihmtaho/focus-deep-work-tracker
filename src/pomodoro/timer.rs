use std::time::Instant;

use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;

use crate::db::{pomodoro_store, session_store};
use crate::models::pomodoro::PomodoroPhase;
use crate::pomodoro::config::PomodoroConfig;

/// Events emitted by `PomodoroTimer::tick_secs`.
#[derive(Debug, Clone)]
pub enum TimerEvent {
    Tick,
    PhaseComplete {
        from: PomodoroPhase,
        to: PomodoroPhase,
        work_saved: bool,
    },
    AutoAbandoned {
        completed: u32,
    },
    Paused,
    Resumed,
}

pub struct PomodoroTimer {
    pub phase: PomodoroPhase,
    pub completed_work: u32,
    pub remaining_secs: u64,
    pub paused: bool,
    /// Accumulated pause seconds across all pause periods in the current phase.
    pub pause_accumulated_secs: u64,
    /// Monotonic instant when the current pause began (None if not paused).
    pub pause_started: Option<Instant>,
    pub config: PomodoroConfig,
    pub task: String,
    pub tag: Option<String>,
    /// Wall-clock start of the overall session (for total-elapsed display).
    pub session_started_at: i64,
}

impl PomodoroTimer {
    pub fn new(task: String, tag: Option<String>, config: PomodoroConfig) -> Self {
        let remaining_secs = (config.work_duration_mins as u64) * 60;
        Self {
            phase: PomodoroPhase::Work,
            completed_work: 0,
            remaining_secs,
            paused: false,
            pause_accumulated_secs: 0,
            pause_started: None,
            config,
            task,
            tag,
            session_started_at: Utc::now().timestamp(),
        }
    }

    /// Advance the timer by `delta_secs` seconds.
    /// Saves completed work phases to the DB and updates stats.
    /// Returns a list of events that occurred during this tick.
    pub fn tick_secs(&mut self, delta_secs: u64, conn: &Connection) -> Result<Vec<TimerEvent>> {
        if delta_secs == 0 {
            return Ok(vec![TimerEvent::Tick]);
        }

        if self.paused {
            // Accumulate pause time and check auto-abandon threshold.
            let paused_so_far = self.pause_accumulated_secs
                + self
                    .pause_started
                    .map(|s| s.elapsed().as_secs())
                    .unwrap_or(0)
                + delta_secs;

            if paused_so_far >= 3600 {
                let completed = self.completed_work;
                self.paused = false;
                return Ok(vec![TimerEvent::AutoAbandoned { completed }]);
            }
            self.pause_accumulated_secs += delta_secs;
            return Ok(vec![TimerEvent::Tick]);
        }

        let mut events = Vec::new();
        let mut remaining_delta = delta_secs;

        // Consume delta, possibly crossing multiple phase boundaries.
        while remaining_delta > 0 {
            if remaining_delta < self.remaining_secs {
                self.remaining_secs -= remaining_delta;
                remaining_delta = 0;
                events.push(TimerEvent::Tick);
            } else {
                // Phase complete.
                remaining_delta = remaining_delta.saturating_sub(self.remaining_secs);
                let completed_phase = self.phase.clone();
                let work_saved = self.advance_phase(conn)?;
                let next_phase = self.phase.clone();
                events.push(TimerEvent::PhaseComplete {
                    from: completed_phase,
                    to: next_phase,
                    work_saved,
                });
            }
        }

        Ok(events)
    }

    /// Transition to the next phase. Returns true if a work session was saved.
    fn advance_phase(&mut self, conn: &Connection) -> Result<bool> {
        let work_saved = match self.phase {
            PomodoroPhase::Work => {
                // Save completed work-phase session record.
                let now = Utc::now().timestamp();
                let work_secs = (self.config.work_duration_mins as i64) * 60;
                let start_epoch = now - work_secs;
                session_store::insert_session_with_times(
                    conn,
                    &self.task,
                    self.tag.as_deref(),
                    "pomodoro",
                    start_epoch,
                    now,
                )?;
                let date = pomodoro_store::today_local_date();
                pomodoro_store::increment_completed(
                    conn,
                    &date,
                    self.config.work_duration_mins,
                    0,
                )?;
                self.completed_work += 1;
                true
            }
            PomodoroPhase::Break | PomodoroPhase::LongBreak => {
                // Record break minutes in stats.
                let date = pomodoro_store::today_local_date();
                let break_mins = match self.phase {
                    PomodoroPhase::Break => self.config.break_duration_mins,
                    PomodoroPhase::LongBreak => self.config.long_break_duration_mins,
                    _ => 0,
                };
                // Update the most recent stat row with break time.
                // We use a separate upsert so break time doesn't double-count.
                conn.execute(
                    "INSERT INTO pomodoro_stats(date, break_minutes) VALUES(?1, ?2) \
                     ON CONFLICT(date) DO UPDATE SET break_minutes = break_minutes + excluded.break_minutes",
                    rusqlite::params![date, break_mins],
                )?;
                false
            }
        };

        // Determine next phase.
        self.phase = match self.phase {
            PomodoroPhase::Work => {
                if self
                    .completed_work
                    .is_multiple_of(self.config.long_break_after)
                {
                    PomodoroPhase::LongBreak
                } else {
                    PomodoroPhase::Break
                }
            }
            PomodoroPhase::Break | PomodoroPhase::LongBreak => PomodoroPhase::Work,
        };

        // Reset remaining time for the new phase.
        self.remaining_secs = match self.phase {
            PomodoroPhase::Work => (self.config.work_duration_mins as u64) * 60,
            PomodoroPhase::Break => (self.config.break_duration_mins as u64) * 60,
            PomodoroPhase::LongBreak => (self.config.long_break_duration_mins as u64) * 60,
        };
        self.pause_accumulated_secs = 0;

        Ok(work_saved)
    }

    pub fn pause(&mut self) {
        if !self.paused {
            self.paused = true;
            self.pause_started = Some(Instant::now());
        }
    }

    pub fn resume(&mut self) {
        if self.paused {
            if let Some(started) = self.pause_started.take() {
                self.pause_accumulated_secs += started.elapsed().as_secs();
            }
            self.paused = false;
        }
    }

    /// Skip the current break phase, transitioning immediately to the next work phase.
    /// No-op if already in a work phase.
    pub fn skip_break(&mut self) {
        if !self.phase.is_work() {
            self.phase = PomodoroPhase::Work;
            self.remaining_secs = (self.config.work_duration_mins as u64) * 60;
            self.pause_accumulated_secs = 0;
        }
    }

    /// Extend the current phase by 5 minutes.
    pub fn extend(&mut self) {
        self.remaining_secs += 300;
    }

    pub fn is_in_work_phase(&self) -> bool {
        self.phase.is_work()
    }

    /// Format remaining seconds as "MM:SS".
    pub fn format_remaining(&self) -> String {
        format_secs(self.remaining_secs)
    }

    /// Total elapsed time since the session started.
    pub fn total_elapsed_secs(&self) -> u64 {
        let now = Utc::now().timestamp();
        (now - self.session_started_at).max(0) as u64
    }

    /// Progress ratio for the current phase (0.0 to 1.0).
    pub fn phase_progress(&self) -> f64 {
        let total = match self.phase {
            PomodoroPhase::Work => (self.config.work_duration_mins as u64) * 60,
            PomodoroPhase::Break => (self.config.break_duration_mins as u64) * 60,
            PomodoroPhase::LongBreak => (self.config.long_break_duration_mins as u64) * 60,
        };
        if total == 0 {
            return 1.0;
        }
        let elapsed = total.saturating_sub(self.remaining_secs);
        (elapsed as f64 / total as f64).clamp(0.0, 1.0)
    }

    pub fn phase_label(&self) -> &str {
        self.phase.label()
    }
}

pub fn format_secs(secs: u64) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{:02}:{:02}", m, s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use tempfile::NamedTempFile;

    fn test_config(work_mins: u32, break_mins: u32) -> PomodoroConfig {
        PomodoroConfig {
            work_duration_mins: work_mins,
            break_duration_mins: break_mins,
            long_break_duration_mins: 15,
            long_break_after: 4,
        }
    }

    fn test_db() -> (Connection, NamedTempFile) {
        let f = NamedTempFile::new().unwrap();
        let conn = open_db_at(f.path()).unwrap();
        (conn, f)
    }

    #[test]
    fn initial_state_is_work_phase() {
        let cfg = test_config(25, 5);
        let t = PomodoroTimer::new("task".into(), None, cfg);
        assert_eq!(t.phase, PomodoroPhase::Work);
        assert_eq!(t.remaining_secs, 25 * 60);
        assert!(!t.paused);
        assert_eq!(t.completed_work, 0);
    }

    #[test]
    fn tick_decrements_remaining() {
        let (conn, _f) = test_db();
        let cfg = test_config(25, 5);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        let events = t.tick_secs(10, &conn).unwrap();
        assert_eq!(t.remaining_secs, 25 * 60 - 10);
        assert!(matches!(events[0], TimerEvent::Tick));
    }

    #[test]
    fn tick_completes_work_phase_transitions_to_break() {
        let (conn, _f) = test_db();
        let cfg = test_config(1, 1); // 1-minute phases for speed
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        let events = t.tick_secs(60, &conn).unwrap();
        let phase_complete = events.iter().any(|e| {
            matches!(
                e,
                TimerEvent::PhaseComplete {
                    from: PomodoroPhase::Work,
                    to: PomodoroPhase::Break,
                    work_saved: true,
                }
            )
        });
        assert!(phase_complete, "expected PhaseComplete Work→Break");
        assert_eq!(t.phase, PomodoroPhase::Break);
        assert_eq!(t.completed_work, 1);
    }

    #[test]
    fn fourth_work_completion_transitions_to_long_break() {
        let (conn, _f) = test_db();
        let cfg = test_config(1, 1);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        // Complete 3 work + 3 break phases to reach the 4th work phase.
        for _ in 0..3 {
            t.tick_secs(60, &conn).unwrap(); // complete work
            t.tick_secs(60, &conn).unwrap(); // complete break
        }
        assert_eq!(t.completed_work, 3);
        assert_eq!(t.phase, PomodoroPhase::Work);
        // Now complete the 4th work phase.
        let events = t.tick_secs(60, &conn).unwrap();
        let long_break = events.iter().any(|e| {
            matches!(
                e,
                TimerEvent::PhaseComplete {
                    to: PomodoroPhase::LongBreak,
                    ..
                }
            )
        });
        assert!(long_break, "4th completion should trigger long break");
        assert_eq!(t.phase, PomodoroPhase::LongBreak);
    }

    #[test]
    fn pause_freezes_remaining_secs() {
        let (conn, _f) = test_db();
        let cfg = test_config(25, 5);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        t.tick_secs(5, &conn).unwrap();
        let before_pause = t.remaining_secs;
        t.pause();
        t.tick_secs(5, &conn).unwrap(); // should not decrement
        assert_eq!(t.remaining_secs, before_pause);
        assert!(t.paused);
    }

    #[test]
    fn resume_continues_from_paused_value() {
        let (conn, _f) = test_db();
        let cfg = test_config(25, 5);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        t.tick_secs(10, &conn).unwrap();
        let before_pause = t.remaining_secs;
        t.pause();
        t.tick_secs(5, &conn).unwrap(); // should not decrement while paused
        t.resume();
        assert!(!t.paused);
        t.tick_secs(1, &conn).unwrap();
        assert_eq!(t.remaining_secs, before_pause - 1);
    }

    #[test]
    fn auto_abandon_after_3600_paused_secs() {
        let (conn, _f) = test_db();
        let cfg = test_config(25, 5);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        t.pause();
        // Tick with delta > 3600 while paused.
        let events = t.tick_secs(3601, &conn).unwrap();
        let abandoned = events
            .iter()
            .any(|e| matches!(e, TimerEvent::AutoAbandoned { .. }));
        assert!(abandoned, "expected AutoAbandoned event");
    }

    #[test]
    fn skip_break_transitions_to_work() {
        let (conn, _f) = test_db();
        let cfg = test_config(1, 5);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        t.tick_secs(60, &conn).unwrap(); // complete work → now in Break
        assert_eq!(t.phase, PomodoroPhase::Break);
        t.skip_break();
        assert_eq!(t.phase, PomodoroPhase::Work);
        assert_eq!(t.remaining_secs, 60); // reset to full work duration
    }

    #[test]
    fn skip_break_in_work_phase_is_noop() {
        let (_conn, _f) = test_db();
        let cfg = test_config(25, 5);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        assert_eq!(t.phase, PomodoroPhase::Work);
        t.skip_break(); // no-op
        assert_eq!(t.phase, PomodoroPhase::Work);
    }

    #[test]
    fn extend_adds_300_secs() {
        let (conn, _f) = test_db();
        let cfg = test_config(25, 5);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        t.tick_secs(60, &conn).unwrap();
        let before = t.remaining_secs;
        t.extend();
        assert_eq!(t.remaining_secs, before + 300);
    }

    #[test]
    fn format_remaining_mm_ss() {
        let cfg = test_config(25, 5);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        t.remaining_secs = 1499; // 24:59
        assert_eq!(t.format_remaining(), "24:59");
        t.remaining_secs = 0;
        assert_eq!(t.format_remaining(), "00:00");
    }

    #[test]
    fn phase_label_contains_emoji() {
        let cfg = test_config(25, 5);
        let t = PomodoroTimer::new("task".into(), None, cfg);
        assert!(t.phase_label().contains("WORK"));
    }

    #[test]
    fn is_in_work_phase() {
        let (conn, _f) = test_db();
        let cfg = test_config(1, 5);
        let mut t = PomodoroTimer::new("task".into(), None, cfg);
        assert!(t.is_in_work_phase());
        t.tick_secs(60, &conn).unwrap();
        assert!(!t.is_in_work_phase());
    }
}
