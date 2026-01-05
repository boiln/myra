use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyraError {
    /// Error from `WinDivert` operations
    #[error("WinDivert error: {0}")]
    WinDivert(#[from] windivert::error::WinDivertError),

    /// Error when a mutex/rwlock is poisoned
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    /// Error when acquiring a statistics lock fails
    #[error("Failed to acquire statistics lock: {0}")]
    StatisticsLock(String),

    /// I/O errors from file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A convenient Result type alias using `MyraError`.
pub type Result<T> = std::result::Result<T, MyraError>;

impl MyraError {
    /// Creates a new lock poisoned error with a descriptive message.
    pub fn lock_poisoned(resource: &str) -> Self {
        Self::LockPoisoned(format!("Failed to acquire lock on {}", resource))
    }

    /// Creates a new statistics lock error.
    pub fn stats_lock(module: &str) -> Self {
        Self::StatisticsLock(format!("{} statistics", module))
    }
}

/// Convert `MyraError` to a String for Tauri command responses.
impl From<MyraError> for String {
    fn from(error: MyraError) -> Self {
        error.to_string()
    }
}
