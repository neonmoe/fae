//! Fae is a small 2D graphics rendering crate, with the main intended
//! use-case being 2D games. Its main goals are simplicity,
//! performance, and compatiblity.

#![warn(missing_docs)]

pub use gl;

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
