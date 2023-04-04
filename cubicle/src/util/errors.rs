use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserApiError {
    #[error("browser's return value doesn't match the standard, {message}")]
    StandardMismatch { message: String }
}
