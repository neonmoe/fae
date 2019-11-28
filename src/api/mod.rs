mod draw_call_handle;
#[cfg(feature = "text")]
mod font_handle;
mod graphics_context;
mod window;

pub use draw_call_handle::{DrawCallHandle, DrawCallParameters};
#[cfg(feature = "text")]
pub use font_handle::FontHandle;
pub use graphics_context::GraphicsContext;
pub use window::Window;

// Re-exports from other parts of the crate
#[cfg(feature = "png")]
pub use crate::error::ImageLoadingError;
pub use crate::gl_version::{OpenGlApi, OpenGlVersion};
pub use crate::image::Image;
pub use crate::sprite::Sprite;
#[cfg(feature = "text")]
pub use crate::text::{Alignment, Text};
pub use crate::types::{Rect, RectPx};

// Re-export Glutin
pub use glutin;
