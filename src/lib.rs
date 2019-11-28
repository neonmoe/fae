//! Fae is a small 2D graphics rendering crate. Its main goals are
//! simplicity, performance, and compatiblity, and so it is a
//! relatively thin layer over OpenGL (2.1/3.3), though the API is a
//! lot simpler. See the `basic` example for a quick overview of the
//! crate's usage.

#![warn(missing_docs)]

#[allow(missing_docs, unknown_lints, clippy::all)]
#[allow(bare_trait_objects)] // Only needed until gl_generator update
pub mod gl {
    //! OpenGL functions and constants.
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod api;
mod error;
mod gl_version;
mod image;
mod renderer;
mod shaders;
mod sprite;
#[cfg(feature = "text")]
mod text;
mod types;

pub mod profiler;
pub use api::*;
