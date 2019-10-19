//! Fae is a small 2D graphics rendering crate. Its main goals are
//! simplicity, performance, and compatiblity, and so it is a
//! relatively thin layer over OpenGL (2.1/3.3), though the API is a
//! lot simpler. See the `basic` example for a quick overview of the
//! crate's usage.

#![warn(missing_docs)]

#[allow(missing_docs, unknown_lints, clippy::all)]
#[allow(bare_trait_objects)] // Only needed until gl_generator update
pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod image;
mod renderer;
mod window;

pub use image::*;
pub use renderer::*;
pub use window::*;

#[cfg(not(feature = "text"))]
mod text_dummy;

#[cfg(feature = "text")]
mod text_rusttype;

/// Text rendering functionality.
pub mod text {
    /// Defines the alignment of text.
    #[derive(Clone, Copy, Debug)]
    pub enum Alignment {
        /// Text is aligned to the left.
        Left,
        /// Text is aligned to the right.
        Right,
        /// Text is centered.
        Center,
    }

    #[cfg(not(feature = "text"))]
    pub use crate::text_dummy::*;
    #[cfg(feature = "text")]
    pub use crate::text_rusttype::*;
}
