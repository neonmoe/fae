//! The error types that `fae` can return.

/// Describes different reasons for why a certain glyph was not drawn.
#[derive(Debug)]
pub(crate) enum GlyphNotRenderedError {
    /// The glyph cache texture could not fit the glyph being
    /// rendered. This is usually caused by trying to draw too
    /// high-resolution text.
    GlyphCacheFull,
}

#[cfg(feature = "png")]
pub use png_errors::*;
#[cfg(feature = "png")]
mod png_errors {
    use std::error::Error;
    use std::fmt;

    /// Describes errors related to parsing image files.
    #[derive(Debug)]
    pub enum ImageLoadingError {
        /// Only RGB and RGBA images are supported.
        UnsupportedFormat(png::ColorType),
        /// Only 8bpc and 16bpc images are supported.
        UnsupportedBitDepth(png::BitDepth),
        /// If the data isn't a valid PNG image, this will describe the
        /// details.
        PngError(png::DecodingError),
    }

    impl fmt::Display for ImageLoadingError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use ImageLoadingError::*;
            match self {
                UnsupportedFormat(color_type) => write!(
                    f,
                    "unsupported color type (not RGB or RGBA): {:?}",
                    color_type
                ),
                UnsupportedBitDepth(bit_depth) => {
                    write!(f, "unsupported bit depth (not 8 or 16): {:?}", bit_depth)
                }
                PngError(err) => err.fmt(f),
            }
        }
    }

    impl Error for ImageLoadingError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                ImageLoadingError::PngError(err) => Some(err),
                _ => None,
            }
        }
    }

    impl From<png::DecodingError> for ImageLoadingError {
        fn from(other: png::DecodingError) -> ImageLoadingError {
            ImageLoadingError::PngError(other)
        }
    }
}
