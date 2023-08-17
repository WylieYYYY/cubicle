//! Error handling and custom error type.

use std::io::ErrorKind;

use thiserror::Error;

/// All possible recoverable errors,
/// may be further separated for better handling.
#[derive(Debug, Error)]
pub enum CustomError {
    // unpredictable system errors
    #[error("input / output error: {0}")]
    IoError(ErrorKind),
    #[error("browser's return value doesn't match the standard, {message}")]
    StandardMismatch { message: String },
    #[error("failed to {verb} container")]
    FailedContainerOperation { verb: String },
    #[error("failed to {verb_prep} storage")]
    FailedStorageOperation { verb_prep: String },
    #[error("failed to fetch the active tab")]
    FailedFetchActiveTab,
    #[error("failed to fetch, {message}")]
    FailedFetchRequest { message: String },
    #[error("failed to {verb} tab")]
    FailedTabOperation { verb: String },

    // predictable errors that are uncommon
    #[error("unsupported version")]
    UnsupportedVersion,

    // predictable errors that are common
    #[error(transparent)]
    InvalidDomain {
        #[from]
        source: idna::Errors,
    },
    #[error("invalid suffix format `{suffix}`")]
    InvalidSuffix { suffix: String },
}
