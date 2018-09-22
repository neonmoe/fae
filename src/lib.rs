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
mod ui;

pub use renderer::*;
pub use ui::*;
