use std::error::Error;
use std::fmt;

/// Describes errors related to parsing image files.
#[derive(Debug)]
pub enum PngLoadingError {
    /// Only 8bpc and 16bpc images are supported.
    UnsupportedBitDepth(png::BitDepth),
    /// If the data isn't a valid PNG image, this will describe the
    /// details.
    PngError(png::DecodingError),
}

impl fmt::Display for PngLoadingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PngLoadingError::*;
        match self {
            UnsupportedBitDepth(bit_depth) => {
                write!(f, "unsupported bit depth (not 8 or 16): {:?}", bit_depth)
            }
            PngError(err) => err.fmt(f),
        }
    }
}

impl Error for PngLoadingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PngLoadingError::PngError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<png::DecodingError> for PngLoadingError {
    fn from(other: png::DecodingError) -> PngLoadingError {
        PngLoadingError::PngError(other)
    }
}
