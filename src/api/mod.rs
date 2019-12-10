#[cfg(feature = "text")]
mod font;
mod graphics_context;
mod spritesheet;
mod window;

#[cfg(feature = "text")]
pub use font::Font;
pub use graphics_context::GraphicsContext;
pub use spritesheet::{Spritesheet, SpritesheetBuilder};
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
