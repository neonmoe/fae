//#![warn(missing_docs)]

#![feature(test)]
#[cfg(test)]
extern crate test;
#[cfg(test)]
mod tests;

#[allow(missing_docs, unknown_lints, clippy::all)]
pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod image;
pub mod renderer;

pub use crate::image::Image;

#[cfg(feature = "text")]
pub mod text;

mod window_settings;

#[cfg(feature = "glfw")]
mod window_glfw;
#[cfg(feature = "glfw")]
pub mod window {
    pub use crate::window_glfw::*;
}

#[cfg(feature = "glutin")]
mod window_glutin;
#[cfg(feature = "glutin")]
pub mod window {
    pub use crate::window_glutin::*;
}

#[cfg(not(any(feature = "glfw", feature = "glutin")))]
mod window_dummy;
#[cfg(not(any(feature = "glfw", feature = "glutin")))]
pub mod window {
    pub use crate::window_dummy::*;
}
