//! The text rendering module.

// NOTE: While the API in this module is still based in logical
// pixels, internally everything should be converted into physical
// pixels as soon as possible. This is to ensure that glyphs end up
// rendered correctly on the actual physical pixels that get
// rasterized.

mod fonts;
mod glyph_cache;
mod layout;
mod text_builder;
pub(crate) mod types;

// This is here for the font8x8_glyphs example
#[cfg(feature = "font8x8")]
#[doc(hidden)]
pub use crate::text::fonts::font8x8::get_bitmap;

pub use self::text_builder::*;
pub use self::types::Alignment;

use crate::error::GlyphNotRenderedError;
use crate::text::glyph_cache::*;
use crate::text::layout::*;
use crate::text::types::*;
use crate::types::*;
use crate::{DrawCallHandle, Renderer};

use std::collections::HashMap;

/// Holds the state required for text rendering, such as the font, and
/// a text draw call queue.
pub struct TextRenderer {
    cache: GlyphCache,
    call: DrawCallHandle,
    glyphs: Vec<Glyph>,
    draw_datas: Vec<TextDrawData>,
    font: Box<dyn FontProvider>,
    dpi_factor: f32,
    // TODO: Add a timer for cached draws to be cleared every now and then?
    draw_cache: HashMap<TextCacheable, (Vec<Glyph>, Option<Rect>)>,
}

impl TextRenderer {
    /// Creates a new text renderer using the font provided by
    /// `font8x8`.
    ///
    /// If `smoothed` is `true`, glyphs which are bigger than 8
    /// physical pixels will be linearly interpolated when stretching
    /// (smooth but blurry). If `false`, nearest-neighbor
    /// interpolation is used (crisp but pixelated).
    #[cfg(feature = "font8x8")]
    pub fn with_font8x8(renderer: &mut Renderer, smoothed: bool) -> TextRenderer {
        let (cache, call) = GlyphCache::create_cache_and_draw_call(renderer, 256, 256, smoothed);
        TextRenderer::with_params(cache, call, Box::new(fonts::Font8x8Provider::new()))
    }

    /// Creates a new text renderer with the given font, rasterized
    /// with `rusttype`.
    #[cfg(feature = "rusttype")]
    pub fn with_ttf(renderer: &mut Renderer, _ttf_data: Vec<u8>) -> TextRenderer {
        let (cache, call) = GlyphCache::create_cache_and_draw_call(renderer, 512, 512, true);

        // TODO: Implement a RustTypeProvider
        TextRenderer::with_params(cache, call, Box::new(fonts::Font8x8Provider::new()))
    }

    fn with_params(
        cache: GlyphCache,
        call: DrawCallHandle,
        font: Box<dyn FontProvider>,
    ) -> TextRenderer {
        TextRenderer {
            cache,
            call,
            glyphs: Vec::new(),
            draw_datas: Vec::new(),
            font,
            dpi_factor: 1.0,
            draw_cache: HashMap::new(),
        }
    }

    /// Updates the DPI multiplication factor of the screen.
    pub fn set_dpi_factor(&mut self, dpi_factor: f32) {
        self.dpi_factor = dpi_factor;
    }

    /// Creates a Sprite struct, which you can render after specifying
    /// your parameters by modifying it.
    ///
    /// ## Optimization tips
    ///
    /// - Set the "cacheability" of the text to `true` with
    ///   [`Text::with_cacheable`](struct.Text.html#with_cacheable) if
    ///   your text (or its parameters) don't change much. Note:
    ///   individual glyphs are always cached. This affects the
    ///   caching of the whole span of text.
    ///
    /// # Usage
    /// ```no_run
    /// # fn main() {
    /// # use fae::{TextRenderer, Text, Renderer, Window, WindowSettings};
    /// # let mut renderer = Renderer::new(&Window::create(&WindowSettings::default()).unwrap());
    /// # let mut text_renderer = TextRenderer::with_font8x8(&mut renderer, true);
    /// text_renderer.draw("Hello, World!", 10.0, 10.0, 0.0, 12.0)
    ///     .with_color((0.8, 0.5, 0.1, 1.0)) // Orange!
    ///     .with_cacheable(true) // Hello world never changes...
    ///     .finish();
    /// # }
    /// ```
    pub fn draw<S: Into<String>>(
        &mut self,
        text: S,
        x: f32,
        y: f32,
        z: f32,
        font_size: f32,
    ) -> Text<'_> {
        let x = (x * self.dpi_factor) as i32;
        let y = (y * self.dpi_factor) as i32;
        let font_size = (font_size * self.dpi_factor) as i32;
        Text::new(self, text.into(), x, y, z, font_size)
    }

    pub(crate) fn draw_text(
        &mut self,
        data: TextCacheable,
        z: f32,
        clip_area: Option<Rect>,
        color: (f32, f32, f32, f32),
        cacheable: bool,
    ) -> Option<Rect> {
        if data.text.is_empty() {
            return None;
        }

        let font_size = data.font_size;

        let draw_data_index = self.draw_datas.len();
        self.draw_datas.push(TextDrawData {
            clip_area,
            color,
            font_size,
            z,
        });

        if let Some((glyphs, bounds)) = self.draw_cache.get(&data) {
            // Append the cached glyphs into the queue if they have been
            // cached, and stop here.
            self.glyphs.extend(glyphs);
            crate::profiler::write(|p| p.layout_cache_hits += glyphs.len() as u32);
            return *bounds;
        }

        let max_line_width = data.max_line_width;
        let (x, y) = (data.x, data.y);
        let alignment = data.alignment;
        let text: &str = &data.text;

        let mut metrics = HashMap::new();
        let mut char_count = 0;
        for c in text.chars() {
            char_count += 1;
            if metrics.get(&c).is_some() {
                continue;
            }

            let glyph_id = if let Some(id) = self
                .font
                .get_glyph_id(c)
                .or_else(|| self.font.get_glyph_id('\u{FFFD}'))
            {
                id
            } else {
                continue;
            };
            let size = self.font.get_metric(glyph_id, font_size);
            let glyph = Metric { glyph_id, size };
            metrics.insert(c, glyph);
        }

        let (mut min_x, mut min_y, mut max_x, mut max_y) = (
            std::f32::INFINITY,
            std::f32::INFINITY,
            std::f32::NEG_INFINITY,
            std::f32::NEG_INFINITY,
        );

        let mut glyphs = Vec::with_capacity(char_count);
        let line_height = self.font.get_line_height(font_size);
        let mut cursor = PositionPx { x, y };
        let mut text_left = text;
        while !text_left.is_empty() {
            let (line_len, line_width) = get_line_length_and_width(
                &*self.font,
                &metrics,
                font_size,
                max_line_width,
                text_left,
            );
            if let Some(max_line_width) = max_line_width {
                cursor.x = get_line_start_x(cursor.x, line_width, max_line_width, alignment);
            }

            let mut previous_character = None;
            let mut chars = text_left.chars();
            for (chars_processed, c) in (&mut chars).enumerate() {
                if let Some(metric) = metrics.get(&c).copied() {
                    // Advance the cursor, if this is not the first character
                    if let Some(previous_character) = previous_character {
                        if let Some(advance) = get_char_advance(
                            &*self.font,
                            &metrics,
                            font_size,
                            c,
                            previous_character,
                        ) {
                            cursor.x += advance;
                        }
                    }

                    let screen_location = metric.size + cursor;
                    min_x = min_x.min(screen_location.x as f32 / self.dpi_factor);
                    min_y = min_y.min(screen_location.y as f32 / self.dpi_factor);
                    max_x = max_x
                        .max((screen_location.x + screen_location.width) as f32 / self.dpi_factor);
                    max_y = max_y
                        .max((screen_location.y + screen_location.height) as f32 / self.dpi_factor);
                    glyphs.push(Glyph {
                        screen_location,
                        draw_data: draw_data_index,
                        id: metric.glyph_id,
                    });
                }
                previous_character = Some(c);

                if chars_processed + 1 >= line_len {
                    break;
                }
            }
            text_left = chars.as_str();

            cursor = (x, cursor.y + line_height).into();
        }
        self.glyphs.extend(&glyphs);

        if let Some((clip_min_x, clip_min_y, clip_max_x, clip_max_y)) =
            clip_area.map(|a| a.into_corners())
        {
            min_x = min_x.max(clip_min_x);
            min_y = min_y.max(clip_min_y);
            max_x = max_x.min(clip_max_x);
            max_y = max_y.min(clip_max_y);
        }

        let bounds = if min_x == std::f32::INFINITY
            || min_y == std::f32::INFINITY
            || max_x == std::f32::NEG_INFINITY
            || max_y == std::f32::NEG_INFINITY
        {
            None
        } else {
            Some((min_x, min_y, max_x - min_x, max_y - min_y).into())
        };

        let len = glyphs.len() as u32;
        crate::profiler::write(|p| p.layout_cache_misses += len);

        if cacheable {
            crate::profiler::write(|p| p.layout_cache_count += len);
            self.draw_cache.insert(data, (glyphs, bounds));
        }

        bounds
    }

    /// Sends all the glyphs to the Renderer. Should be called every
    /// frame before
    /// [`Renderer::render`](../struct.Renderer.html#method.render).
    pub fn compose_draw_call(&mut self, renderer: &mut Renderer) {
        for glyph in &self.glyphs {
            let font_size = self.draw_datas[glyph.draw_data].font_size;
            let color = self.draw_datas[glyph.draw_data].color;
            let z = self.draw_datas[glyph.draw_data].z;

            // Note to reader: Careful attention is paid to the fact
            // that the `screen_location` Rect's and `texcoords`
            // Rect's fields have exactly the same fractional
            // parts. This is to ensure that glyphs are drawn pixel
            // perfectly in the case of matching width & height, which
            // is the case when rendering glyphs that were rasterized
            // specifically for this resolution (ie. not bitmap
            // fonts). An offset is applied (0.5px in each direction,
            // expanding outwards) to capture half a pixel around the
            // glyph: if the glyph texture is stretched, this will
            // preserve the linear blending around the border of the
            // glyph, and does nothing if the texture is not
            // stretched. Note that the glyph cache keeps gaps between
            // glyphs to avoid leaking because of this.

            let screen_location = Rect {
                x: glyph.screen_location.x as f32 - 0.5,
                y: glyph.screen_location.y as f32 - 0.5,
                width: glyph.screen_location.width as f32 + 1.0,
                height: glyph.screen_location.height as f32 + 1.0,
            };

            let mut sprite = renderer
                .draw(&self.call, z)
                .with_physical_coordinates(screen_location)
                .with_color(color.0, color.1, color.2, color.3);
            if let Some(area) = self.draw_datas[glyph.draw_data].clip_area {
                sprite = sprite.with_clip_area(area);
            }

            let finish_sprite = |texcoords: RectPx| {
                let texcoords = Rect {
                    x: texcoords.x as f32 - 0.5,
                    y: texcoords.y as f32 - 0.5,
                    width: texcoords.width as f32 + 1.0,
                    height: texcoords.height as f32 + 1.0,
                };
                sprite.with_texture_coordinates(texcoords).finish();
            };

            match self.font.render_glyph(&mut self.cache, glyph.id, font_size) {
                Ok(texcoords) => finish_sprite(texcoords),
                Err(err) => match err {
                    // TODO: Report this to the crate user somehow
                    GlyphNotRenderedError::GlyphCacheFull => continue,
                    GlyphNotRenderedError::GlyphInvisible => continue,
                },
            }
        }

        self.glyphs.clear();
        self.draw_datas.clear();
        self.cache.expire_one_step();
    }

    /// Draws the glyph cache texture in the given screen-space quad,
    /// for debugging.
    pub fn debug_draw_glyph_cache<R: Into<Rect>>(
        &self,
        renderer: &mut Renderer,
        coordinates: R,
        z: f32,
    ) {
        renderer
            .draw(&self.call, z)
            .with_coordinates(coordinates)
            .with_uvs((0.0, 0.0, 1.0, 1.0))
            .with_color(0.0, 0.0, 0.0, 1.0)
            .finish();
    }
}
