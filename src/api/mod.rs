#[cfg(feature = "text")]
mod font;
mod graphics_context;
mod spritesheet;

#[cfg(feature = "text")]
pub use font::Font;
pub use graphics_context::{Context, GraphicsContext};
pub use spritesheet::{Spritesheet, SpritesheetBuilder};

// Re-exports from other parts of the crate
pub mod errors {
    //! The errors that fae can return.
    pub use crate::error::ImageCreationError;
    #[cfg(feature = "png")]
    pub use crate::error::PngLoadingError;
}
pub use crate::error::Error;
pub use crate::gl_version::{OpenGlApi, OpenGlVersion};
pub use crate::image::Image;
pub use crate::renderer::TextureWrapping;
pub use crate::shaders::{ShaderPair, Shaders};
pub use crate::sprite::Sprite;
#[cfg(feature = "text")]
pub use crate::text::{Alignment, Text};
pub use crate::types::Rect;
