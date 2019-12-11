use std::error::Error;
use std::fmt;

/// Describes errors during the creation of an image.
#[derive(Debug)]
pub enum ImageCreationError {
    /// The color assigned to the image consisted of more than 4
    /// components (only red, green, blue and alpha components are
    /// supported) or no components at all.
    InvalidColorComponentCount(usize),
}

impl fmt::Display for ImageCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageCreationError::InvalidColorComponentCount(count) => {
                write!(f, "unsupported color component count (not 1-4): {}", count)
            }
        }
    }
}

impl Error for ImageCreationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
