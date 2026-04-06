use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: i64,
    pub task: String,
    pub tag: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub mode: String,
}

#[allow(dead_code)]
impl Session {
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    pub fn duration(&self) -> Option<Duration> {
        self.end_time.map(|end| end - self.start_time)
    }

    pub fn elapsed(&self) -> Duration {
        Utc::now() - self.start_time
    }
}
