//! # fungui
//!
//! Work-in-progress GUI crate for making easy-to-code lightweight
//! GUIs. See the README for more information. Since the crate is deep
//! in development, I won't write a general guide to using it yet,
//! aside from the examples.
#![warn(missing_docs)]

extern crate gl;
#[cfg(feature = "window_bootstrap")]
extern crate glutin;
#[macro_use]
extern crate lazy_static;
extern crate png;
extern crate rusttype;
extern crate unicode_normalization;

#[cfg(feature = "window_bootstrap")]
mod window;
#[cfg(feature = "window_bootstrap")]
pub use window::*;

#[cfg(feature = "default_resources")]
pub mod resources;

mod image;
mod renderer;
mod text;
mod ui;

pub use renderer::*;
pub use text::initialize_font;
pub use ui::element;
pub use ui::layout;
