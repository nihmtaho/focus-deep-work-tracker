/// A single Pomodoro phase type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PomodoroPhase {
    Work,
    Break,
    LongBreak,
}

impl PomodoroPhase {
    pub fn label(&self) -> &str {
        match self {
            PomodoroPhase::Work => "🍅 WORK",
            PomodoroPhase::Break => "☕ BREAK",
            PomodoroPhase::LongBreak => "🌿 LONG BREAK",
        }
    }

    pub fn is_work(&self) -> bool {
        matches!(self, PomodoroPhase::Work)
    }
}

/// Per-day aggregate Pomodoro statistics.
#[derive(Debug, Clone, Default)]
pub struct PomodoroStats {
    pub date: String,
    pub completed: u32,
    pub abandoned: u32,
    pub work_minutes: u32,
    pub break_minutes: u32,
}
