use thiserror::Error;

use crate::interop::contextual_identities::ContextualIdentity;

#[derive(Error, Debug)]
pub enum BrowserApiError {
    #[error("browser's return value doesn't match the standard, {message}")]
    StandardMismatch { message: String },
    #[error("failed to delete {container}")]
    FailedContainerDeletion { container: ContextualIdentity },
    #[error("failed to update container `{name}`")]
    FailedContainerUpdate { name: String },
    #[error("failed to create container `{name}`")]
    FailedContainerCreation { name: String }
}
