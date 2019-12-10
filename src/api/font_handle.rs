use crate::api::{DrawCallHandle, GraphicsContext};
use crate::text::{Text, TextRenderer};

/// A handle to a font. Can be used to draw strings of text.
#[cfg(feature = "text")]
#[derive(Clone, Debug)]
pub struct FontHandle {
    /// Points to the GraphicsContext's internal Vec<TextRenderer>.
    index: usize,
}

#[cfg(feature = "text")]
impl FontHandle {
    /// Creates a new font renderer using the given .ttf file as a
    /// font.
    #[cfg(feature = "ttf")]
    pub fn with_ttf(
        ctx: &mut GraphicsContext,
        ttf_data: Vec<u8>,
    ) -> Result<FontHandle, rusttype::Error> {
        let text = TextRenderer::with_ttf(&mut ctx.renderer, ttf_data)?;
        ctx.text_renderers.push(text);
        Ok(FontHandle {
            index: ctx.text_renderers.len() - 1,
        })
    }

    /// Creates a new font renderer using the font8x8 font.
    ///
    /// If `smoothed` is `true`, glyphs which are bigger than 8
    /// physical pixels will be linearly interpolated when stretching
    /// (smooth but blurry). If `false`, nearest-neighbor
    /// interpolation is used (crisp but pixelated).
    #[cfg(feature = "font8x8")]
    pub fn with_font8x8(ctx: &mut GraphicsContext, smoothed: bool) -> FontHandle {
        let text = TextRenderer::with_font8x8(&mut ctx.renderer, smoothed);
        ctx.text_renderers.push(text);
        FontHandle {
            index: ctx.text_renderers.len() - 1,
        }
    }

    /// Creates a Text struct, which you can render after
    /// specifying your parameters by modifying it.
    ///
    /// # Usage
    /// ```ignore
    /// font.draw(&mut ctx, "Hello, World!", 10.0, 10.0, 0.0, 12.0)
    ///     .color((0.8, 0.5, 0.1, 1.0))
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

    /// Returns the underlying draw call of this font. Can be used to
    /// render the glyph cache texture, which could be useful for
    /// debugging.
    pub fn draw_call(&self, ctx: &GraphicsContext) -> DrawCallHandle {
        ctx.text_renderers[self.index].draw_call().clone()
    }
}
