use std::path::PathBuf;

use thiserror::Error;

/// Failure while creating, opening, validating, or saving a portable job.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("file operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("destination already exists: {}", .0.display())]
    DestinationExists(PathBuf),
    #[error("file is not a LibrePAT job database")]
    InvalidApplicationId,
    #[error("schema version {0} is newer than this application supports")]
    UnsupportedSchema(i64),
    #[error("schema version {0} is not supported")]
    InvalidSchema(i64),
    #[error("embedded source data does not match its SHA-256 hash")]
    SourceHashMismatch,
    #[error("electrical results and imported source data are immutable")]
    ImmutableResults,
    #[error("job data is invalid: {0}")]
    InvalidJob(String),
    #[error("job database is corrupt: {0}")]
    Corrupt(String),
    #[error("could not restore the original job after replacement failed: {0}")]
    ReplacementRecovery(String),
}
