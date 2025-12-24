use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyraError {
    /// Error from WinDivert operations
    #[error("WinDivert error: {0}")]
    WinDivert(#[from] windivert::error::WinDivertError),

    /// Error when a mutex/rwlock is poisoned
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    /// Error when acquiring a statistics lock fails
    #[error("Failed to acquire statistics lock: {0}")]
    StatisticsLock(String),

    /// Error in configuration handling
    #[error("Configuration error: {0}")]
    Config(String),

    /// I/O errors from file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error when packet processing is already running
    #[error("Packet processing is already running")]
    AlreadyRunning,

    /// Error when packet processing is not running
    #[error("Packet processing is not running")]
    NotRunning,

    /// Error during packet processing
    #[error("Packet processing error: {0}")]
    Processing(String),

    /// Error when sending packets fails
    #[error("Failed to send packet: {0}")]
    PacketSend(String),

    /// Error during serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Generic error for other cases
    #[error("{0}")]
    Other(String),
}

/// A convenient Result type alias using MyraError.
pub type Result<T> = std::result::Result<T, MyraError>;

impl MyraError {
    /// Creates a new lock poisoned error with a descriptive message.
    pub fn lock_poisoned(resource: &str) -> Self {
        MyraError::LockPoisoned(format!("Failed to acquire lock on {}", resource))
    }

    /// Creates a new statistics lock error.
    pub fn stats_lock(module: &str) -> Self {
        MyraError::StatisticsLock(format!("{} statistics", module))
    }
}

/// Convert MyraError to a String for Tauri command responses.
impl From<MyraError> for String {
    fn from(error: MyraError) -> Self {
        error.to_string()
    }
}

/// Helper trait to convert PoisonError to MyraError.
pub trait LockResultExt<T> {
    /// Convert a lock result to a MyraError result.
    fn map_lock_err(self, resource: &str) -> Result<T>;
}

impl<T, E> LockResultExt<T> for std::result::Result<T, std::sync::PoisonError<E>> {
    fn map_lock_err(self, resource: &str) -> Result<T> {
        self.map_err(|_| MyraError::lock_poisoned(resource))
    }
}
