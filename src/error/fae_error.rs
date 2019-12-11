#[cfg(feature = "png")]
use crate::error::PngLoadingError;
use crate::error::{GlutinError, ImageCreationError};

use std::fmt;

/// A generic error type that wraps fae's other error types in the
/// [`fae::errors`](errors/index.html) module.
///
/// This is a convenience type you can use to represent all errors
/// produced by fae, as any ?'d error can be automatically turned into
/// this type via the From implementation.
///
/// # Example
/// ```no_run
/// // Note: the error type is fae::Error!
/// fn main() -> Result<(), fae::Error> {
///
///     // Produces a fae::errors::GlutinError.
///     let window = fae::Window::new()?;
///
///     // Produces a fae::errors::ImageCreationError.
///     let color_image = fae::Image::with_color(16, 16, &[0xFF])?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub enum Error {
    /// See [`GlutinError`](enum.GlutinError.html).
    GlutinError(GlutinError),
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
            GlutinError(err) => err.fmt(f),
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
            GlutinError(err) => Some(err),
            ImageCreationError(err) => Some(err),
            #[cfg(feature = "png")]
            PngLoadingError(err) => Some(err),
        }
    }
}

impl From<GlutinError> for Error {
    fn from(error: GlutinError) -> Error {
        Error::GlutinError(error)
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
