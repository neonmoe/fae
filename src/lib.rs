//#![warn(missing_docs)]

#[allow(missing_docs, unknown_lints, clippy::all)]
pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod image;
mod rect;
pub mod renderer;

pub use crate::image::Image;
pub use crate::rect::Rect;

#[cfg(feature = "text")]
pub mod text;

#[cfg(feature = "glutin")]
pub mod window;
