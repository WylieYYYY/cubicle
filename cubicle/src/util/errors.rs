use std::io::ErrorKind;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
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

    #[error(transparent)]
    InvalidDomain { #[from] source: idna::Errors },
    #[error("invalid suffix format `{suffix}`")]
    InvalidSuffix { suffix: String }
}
