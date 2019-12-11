//! The error types that `fae` can return.

mod glutin_error;
pub use glutin_error::GlutinError;

mod image_creation_error;
pub use image_creation_error::ImageCreationError;

#[cfg(feature = "png")]
mod image_png_error;
#[cfg(feature = "png")]
pub use image_png_error::PngLoadingError;
