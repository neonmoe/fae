//! The error types that `fae` can return.

mod fae_error;
pub use fae_error::Error;

mod image_creation_error;
pub use image_creation_error::ImageCreationError;

#[cfg(feature = "png")]
mod image_png_error;
#[cfg(feature = "png")]
pub use image_png_error::PngLoadingError;
