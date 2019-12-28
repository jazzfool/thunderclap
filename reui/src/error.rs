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

#[cfg(feature = "default-themes")]
#[derive(Error, Debug)]
pub enum ThemeError {
    #[error("{0}")]
    ResourceError(#[from] error::ResourceError),
    #[error("{0}")]
    FontError(#[from] error::FontError),
}
