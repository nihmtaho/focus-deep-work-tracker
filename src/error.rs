use thiserror::Error;

#[derive(Debug, Error)]
pub enum FocusError {
    #[error("No active session to stop.")]
    NoActiveSession,

    #[error("Session already running: \"{task}\" — elapsed: {elapsed}")]
    AlreadyRunning { task: String, elapsed: String },

    #[error("Task description cannot be empty.")]
    EmptyTask,

    #[error("--limit must be a positive integer.")]
    InvalidLimit,

    #[error("--format must be one of: json, markdown.")]
    InvalidFormat,

    #[error("Data file is corrupted or unreadable: {path}")]
    DataFileCorrupted { path: String },

    #[error("Session #{id} not found.")]
    SessionNotFound { id: i64 },

    #[error("--{field} must be between {min} and {max} minutes (got {value}).")]
    InvalidPomoDuration {
        field: String,
        value: u32,
        min: u32,
        max: u32,
    },

    #[error(transparent)]
    Db(#[from] rusqlite::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
