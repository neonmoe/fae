use std::error::Error;
use std::fmt;

/// Describes errors during window creation.
#[derive(Debug)]
pub enum GlutinError {
    /// [`glutin::Context::make_current`](https://docs.rs/glutin/0.22.0-alpha5/glutin/struct.Context.html#method.make_current)
    /// encountered an error.
    ContextError(glutin::ContextError),
    /// [`glutin::ContextBuilder::build_windowed`](https://docs.rs/glutin/0.22.0-alpha5/glutin/struct.ContextBuilder.html#method.build_windowed)
    /// encountered an error.
    CreationError(glutin::CreationError),
}

impl fmt::Display for GlutinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlutinError::ContextError(err) => err.fmt(f),
            GlutinError::CreationError(err) => err.fmt(f),
        }
    }
}

impl Error for GlutinError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GlutinError::ContextError(err) => Some(err),
            GlutinError::CreationError(err) => Some(err),
        }
    }
}

impl From<glutin::CreationError> for GlutinError {
    fn from(error: glutin::CreationError) -> GlutinError {
        GlutinError::CreationError(error)
    }
}

impl<T> From<(T, glutin::ContextError)> for GlutinError {
    fn from(error: (T, glutin::ContextError)) -> GlutinError {
        GlutinError::ContextError(error.1)
    }
}
