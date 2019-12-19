#[cfg(feature = "app")]
use {reclutch::error, thiserror::Error};

#[cfg(feature = "app")]
#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    SkiaError(#[from] error::SkiaError),
    #[error("{0}")]
    ResourceError(#[from] error::ResourceError),
}
