use crate::error::ImageCreationError;
#[cfg(feature = "png")]
use crate::error::PngLoadingError;

use std::fmt;

/// A generic error type that wraps fae's other error types in the
/// [`fae::errors`](errors/index.html) module.
#[derive(Debug)]
pub enum Error {
    /// See [`ImageCreationError`](enum.ImageCreationError.html).
    ImageCreationError(ImageCreationError),
    /// See [`PngLoadingError`](enum.PngLoadingError.html).
    #[cfg(feature = "png")]
    PngLoadingError(PngLoadingError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            ImageCreationError(err) => err.fmt(f),
            #[cfg(feature = "png")]
            PngLoadingError(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            ImageCreationError(err) => Some(err),
            #[cfg(feature = "png")]
            PngLoadingError(err) => Some(err),
        }
    }
}

impl From<ImageCreationError> for Error {
    fn from(error: ImageCreationError) -> Error {
        Error::ImageCreationError(error)
    }
}

#[cfg(feature = "png")]
impl From<PngLoadingError> for Error {
    fn from(error: PngLoadingError) -> Error {
        Error::PngLoadingError(error)
    }
}
