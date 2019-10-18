//! Fae is a small 2D graphics rendering crate. Its main goals are
//! simplicity, performance, and compatiblity, and so it is a
//! relatively thin layer over OpenGL (2.1/3.3), though the API is a
//! lot simpler. See the `basic` example for a quick overview of the
//! crate's usage.

#![warn(missing_docs)]

#[allow(missing_docs, unknown_lints, clippy::all)]
pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod image;
pub mod renderer;

pub use crate::image::Image;

#[cfg(feature = "text")]
pub mod text;

#[cfg(feature = "glutin")]
pub mod window;
