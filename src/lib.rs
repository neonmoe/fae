//! Fae is a small 2D graphics rendering crate, with the main intended
//! use-case being 2D games. Its main goals are simplicity,
//! performance, and compatiblity. See the `basic` example for a quick
//! overview of the crate's usage.

#![warn(missing_docs)]

#[allow(missing_docs, unknown_lints, clippy::all)]
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
