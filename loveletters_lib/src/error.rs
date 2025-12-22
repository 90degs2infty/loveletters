use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An unspecified error
    #[error("unspecified internal error occurred")]
    Other,
}

pub type Result<T> = std::result::Result<T, Error>;
