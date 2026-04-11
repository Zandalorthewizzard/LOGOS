use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum JournalError {
    #[error("failed to create journal directory '{path}': {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to append journal record to '{path}': {source}")]
    Append {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read journal '{path}': {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse journal record in '{path}': {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("journal session '{session_id}' is missing SessionOpened record")]
    MissingSessionOpened { session_id: String },
}
