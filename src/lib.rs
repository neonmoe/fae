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
mod renderable;
mod renderer;
mod window;

pub use image::*;
pub use renderable::*;
pub use renderer::*;
pub use window::*;

#[cfg(feature = "text")]
pub mod text;

/// Module for reading profiling data that `fae` collects. If the
/// `profiler` feature is disabled, all the functions are no-ops, and
/// there will be no profiling overhead.
pub mod profiler;
