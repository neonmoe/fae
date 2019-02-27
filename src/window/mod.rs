//! Quick window creation utilities.

mod mouse;
mod util;
#[cfg(not(any(feature = "glfw", feature = "glutin")))]
mod window_dummy;
#[cfg(feature = "glfw")]
mod window_glfw;
#[cfg(feature = "glutin")]
mod window_glutin;

pub use mouse::*;
pub use util::*;
#[cfg(not(any(feature = "glfw", feature = "glutin")))]
pub use window_dummy::*;
#[cfg(feature = "glfw")]
pub use window_glfw::*;
#[cfg(feature = "glutin")]
pub use window_glutin::*;
