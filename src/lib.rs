extern crate gl;
#[cfg(feature = "window_bootstrap")]
extern crate glutin;
extern crate png;

#[cfg(feature = "window_bootstrap")]
mod window;
#[cfg(feature = "window_bootstrap")]
pub use window::Window;

mod image;
mod renderer;
pub use renderer::{draw_quad, initialize, render};
