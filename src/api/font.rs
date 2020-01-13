use crate::api::{GraphicsContext, Spritesheet};
use crate::text::{Text, TextRenderer};

/// Holds a font for rendering. See also:
/// [`Font::draw`](struct.Font.html#method.draw).
#[derive(Clone, Debug)]
pub struct Font {
    /// Points to the GraphicsContext's internal Vec<TextRenderer>.
    index: usize,
    /// Contains the draw call handle used by the TextRenderer.
    spritesheet: Spritesheet,
}

impl Font {
    /// Creates a new font renderer using the given .ttf file as a
    /// font.
    ///
    /// The font is rasterized with
    /// [`rusttype`](https://crates.io/crates/rusttype).
    #[cfg(feature = "ttf")]
    pub fn with_ttf(ctx: &mut GraphicsContext, ttf_data: Vec<u8>) -> Result<Font, rusttype::Error> {
        let text = TextRenderer::with_ttf(&mut ctx.renderer, ttf_data)?;
        let handle = text.draw_call().clone();
        ctx.text_renderers.push(text);
        Ok(Font {
            index: ctx.text_renderers.len() - 1,
            spritesheet: Spritesheet { handle },
        })
    }

    /// Creates a new font renderer using the
    /// [`font8x8`](https://crates.io/crates/font8x8) font.
    ///
    /// If `smoothed` is `true`, glyphs which are bigger than 8
    /// physical pixels will be linearly interpolated when stretching
    /// (smooth but blurry). If `false`, nearest-neighbor
    /// interpolation is used (crisp but pixelated).
    #[cfg(feature = "font8x8")]
    pub fn with_font8x8(ctx: &mut GraphicsContext, smoothed: bool) -> Font {
        let text = TextRenderer::with_font8x8(&mut ctx.renderer, smoothed);
        let handle = text.draw_call().clone();
        ctx.text_renderers.push(text);
        Font {
            index: ctx.text_renderers.len() - 1,
            spritesheet: Spritesheet { handle },
        }
    }

    /// Creates a Text struct, which you can render after
    /// specifying your parameters by modifying it.
    ///
    /// # Usage
    /// ```no_run
    /// # let mut ctx = fae::GraphicsContext::dummy();
    /// // Initialize the font once somewhere, usually before the game loop:
    /// let font = fae::Font::with_font8x8(&mut ctx, true);
    ///
    /// // Then in rendering code, call draw:
    /// font.draw(&mut ctx, "Hello, World!", 10.0, 10.0, 12.0)
    ///     .color((0.8, 0.5, 0.1, 1.0))
    ///     .finish();
    /// ```
    pub fn draw<'a, S: Into<String>>(
        &self,
        ctx: &'a mut GraphicsContext,
        text: S,
        x: f32,
        y: f32,
        font_size: f32,
    ) -> Text<'a> {
        ctx.text_renderers[self.index].draw(text.into(), x, y, font_size)
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
    /// render the glyph cache texture, which is useful for debugging
    /// rasterization, glyph rendering, or other glyph-cache-related
    /// problems.
    pub fn spritesheet(&self) -> &Spritesheet {
        &self.spritesheet
    }
}
