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

#[cfg(feature = "text")]
pub mod text;

// TODO: Add a feature for using the font8x8 crate as a font
// text_dummy -> text_renderer, uses a font provider + font8x8 / font-kit as font providers
