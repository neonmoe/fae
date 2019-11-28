use crate::gl_version::OpenGlVersion;
use crate::renderer::{DrawCallHandle, DrawCallParameters, Renderer};
#[cfg(feature = "text")]
use crate::text::{Text, TextRenderer};

use glutin::dpi::LogicalSize;
use glutin::{PossiblyCurrent, WindowedContext};

/// The graphics context: used to draw stuff on the screen.
///
/// Borrow this struct from:
/// - [`Window::ctx`](struct.Window.html#field.ctx) before starting
///   the event loop,
/// - The first parameter of the closure in
///   [`Window::run`](struct.Window.html#method.run) (during the event
///   loop).
///
/// Then, pass it to:
/// - [`DrawCallHandle::draw`](struct.DrawCallHandle.html#method.draw) to draw sprites,
/// - [`FontHandle::draw`](struct.FontHandle.html#method.draw) to draw text.
pub struct GraphicsContext {
    #[cfg(feature = "text")]
    pub(crate) text_renderers: Vec<TextRenderer>,
    pub(crate) window: WindowedContext<PossiblyCurrent>,
    pub(crate) renderer: Renderer,
    pub(crate) env_dpi_factor: f32,

    /// The width of the window in logical coordinates. Multiply with
    /// `dpi_factor` to get the width in physical pixels.
    pub width: f32,
    /// The height of the window in logical coordinates. Multiply with
    /// `dpi_factor` to get the height in physical pixels.
    pub height: f32,
    /// The dpi multiplier of the window.
    pub dpi_factor: f32,
}

impl GraphicsContext {
    /// Updates the window (swaps the front and back buffers).
    pub(crate) fn swap_buffers(&mut self) {
        let _ = self.window.swap_buffers();
        self.renderer.synchronize();
    }

    pub(crate) fn resize(&mut self, logical_size: LogicalSize, dpi_factor: f64) {
        let physical_size = logical_size.to_physical(dpi_factor);
        let (width, height): (u32, u32) = physical_size.into();
        unsafe {
            crate::gl::Viewport(0, 0, width as i32, height as i32);
        }
        self.window.resize(physical_size);
        self.width = logical_size.width as f32 / self.env_dpi_factor;
        self.height = logical_size.height as f32 / self.env_dpi_factor;
        self.dpi_factor = dpi_factor as f32 * self.env_dpi_factor;
    }
}

/// Creation utilities for various handles.
impl GraphicsContext {
    /// Creates a new draw call, and returns a handle to it. You can draw with the handle.
    pub fn create_draw_call(&mut self, params: DrawCallParameters) -> DrawCallHandle {
        self.renderer.create_draw_call(params)
    }

    /// Creates a new font renderer (using the given TTF file as a
    /// font) and returns a handle to it.
    #[cfg(all(feature = "text", feature = "ttf"))]
    pub fn create_font_ttf(&mut self, ttf_data: Vec<u8>) -> Result<FontHandle, rusttype::Error> {
        let text = TextRenderer::from_ttf(&mut self.renderer, ttf_data)?;
        self.text_renderers.push(text);
        Ok(FontHandle {
            index: self.text_renderers.len() - 1,
        })
    }

    /// Creates a new font renderer (using the font8x8 font) and
    /// returns a handle to it.
    ///
    /// If `smoothed` is `true`, glyphs which are bigger than 8
    /// physical pixels will be linearly interpolated when stretching
    /// (smooth but blurry). If `false`, nearest-neighbor
    /// interpolation is used (crisp but pixelated).
    #[cfg(all(feature = "text", feature = "font8x8"))]
    pub fn create_font8x8(&mut self, smoothed: bool) -> FontHandle {
        let text = TextRenderer::from_font8x8(&mut self.renderer, smoothed);
        self.text_renderers.push(text);
        FontHandle {
            index: self.text_renderers.len() - 1,
        }
    }
}

/// Metadata about the OpenGL context.
impl GraphicsContext {
    /// Returns whether or not running in legacy mode (OpenGL 3.3+
    /// optimizations off).
    pub fn is_legacy(&self) -> bool {
        self.renderer.legacy
    }

    /// Returns the OpenGL version if it could be parsed.
    pub fn get_opengl_version(&self) -> &OpenGlVersion {
        &self.renderer.version
    }
}

/// A handle to a font. Can be used to draw strings of text.
///
/// Created with
/// [`GraphicsContext::create_font_ttf`](struct.GraphicsContext.html#method.create_font_ttf)
/// or
/// [`GraphicsContext::create_font8x8`](struct.GraphicsContext.html#method.create_font8x8)
/// depending on your needs.
#[cfg(feature = "text")]
#[derive(Clone, Debug)]
pub struct FontHandle {
    pub(crate) index: usize,
}

#[cfg(feature = "text")]
impl FontHandle {
    /// Creates a Text struct, which you can render after
    /// specifying your parameters by modifying it.
    ///
    /// # Usage
    /// ```ignore
    /// font.draw(&mut ctx, "Hello, World!", 10.0, 10.0, 0.0, 12.0)
    ///     .with_color((0.8, 0.5, 0.1, 1.0))
    ///     .finish();
    /// ```
    pub fn draw<'a, S: Into<String>>(
        &mut self,
        ctx: &'a mut GraphicsContext,
        text: S,
        x: f32,
        y: f32,
        z: f32,
        font_size: f32,
    ) -> Text<'a> {
        ctx.text_renderers[self.index].draw(text.into(), x, y, z, font_size)
    }

    /// Returns true if this font failed to draw a glyph last
    /// frame because the glyph cache was full. Generally, this
    /// should become false on the next frame because the glyph
    /// cache is resized at the start of the frame, as
    /// needed. Resizing is limited by `GL_MAX_TEXTURE_SIZE`
    /// however, so low-end systems might reach a limit if you're
    /// drawing lots of very large text using many symbols.
    ///
    /// # What to do if the glyph cache is full
    ///
    /// Consider using alternative means of rendering large text, or
    /// increase your application's GPU capability requirements.
    pub fn is_glyph_cache_full(&self, ctx: &GraphicsContext) -> bool {
        ctx.text_renderers[self.index].glyph_cache_filled
    }

    /// Draws the glyph cache texture in the given quad, for
    /// debugging.
    pub fn debug_draw_glyph_cache<R: Into<crate::types::Rect>>(
        &self,
        ctx: &mut GraphicsContext,
        coordinates: R,
        z: f32,
    ) {
        ctx.text_renderers[self.index].debug_draw_glyph_cache(
            &mut ctx.renderer,
            coordinates.into(),
            z,
        )
    }
}
