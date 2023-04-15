use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("browser's return value doesn't match the standard, {message}")]
    StandardMismatch { message: String },
    #[error("failed to delete container")]
    FailedContainerDeletion,
    #[error("failed to update container `{name}`")]
    FailedContainerUpdate { name: String },
    #[error("failed to create container `{name}`")]
    FailedContainerCreation { name: String },

    #[error("container is locked by `{locker}`")]
    ContainerLocked { locker: String },
    #[error(transparent)]
    InvalidDomain { #[from] source: idna::Errors },
    #[error("invalid suffix format `{suffix}`")]
    InvalidSuffix { suffix: String }
}
