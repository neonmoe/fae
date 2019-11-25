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

pub use self::text_builder::Text;
pub use self::types::Alignment;

use self::text_builder::TextCacheable;
use crate::error::GlyphNotRenderedError;
use crate::text::glyph_cache::*;
use crate::text::layout::*;
use crate::text::types::*;
use crate::types::*;
use crate::{DrawCallHandle, Renderer};

use std::collections::HashMap;

/// Contains everything needed to draw text.
///
/// Holds a HashMap to cache draws marked as cacheable, which takes up
/// roughly 33 bytes per draw, 37 bytes per glyph, and whatever
/// overhead HashMap has. (Note: these values are probably not
/// accurate, and were eyeballed at the time of writing.)
pub struct TextRenderer {
    cache: GlyphCache,
    glyph_cache_filled: bool,
    call: DrawCallHandle,
    glyphs: Vec<Glyph>,
    draw_datas: Vec<TextDrawData>,
    font: Box<dyn FontProvider>,
    dpi_factor: f32,
    window_size: (f32, f32),
    glyph_ids: HashMap<char, GlyphId>,
    // TODO(optimization): Unused cached values should be dropped (layout cache)
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
        let (cache, call) = GlyphCache::create_cache_and_draw_call(renderer, 64, 64, smoothed);
        TextRenderer::with_params(cache, call, Box::new(fonts::Font8x8Provider::new()))
    }

    /// Creates a new text renderer with the given font, rasterized
    /// with `rusttype`.
    #[cfg(feature = "ttf")]
    pub fn with_ttf(
        renderer: &mut Renderer,
        ttf_data: Vec<u8>,
    ) -> Result<TextRenderer, rusttype::Error> {
        let (cache, call) = GlyphCache::create_cache_and_draw_call(renderer, 256, 256, true);
        let font = Box::new(fonts::RustTypeProvider::from_ttf(ttf_data)?);
        Ok(TextRenderer::with_params(cache, call, font))
    }

    fn with_params(
        cache: GlyphCache,
        call: DrawCallHandle,
        font: Box<dyn FontProvider>,
    ) -> TextRenderer {
        TextRenderer {
            cache,
            glyph_cache_filled: false,
            call,
            glyphs: Vec::new(),
            draw_datas: Vec::new(),
            font,
            dpi_factor: 1.0,
            window_size: (0.0, 0.0),
            glyph_ids: HashMap::new(),
            draw_cache: HashMap::new(),
        }
    }

    /// Returns true if the last
    /// [`compose_draw_call`](struct.TextRenderer.html#method.compose_draw_call)
    /// failed to draw a glyph because the glyph cache was full and
    /// could not be expanded to fit the glyph. Note that this should
    /// be quite rare, unless you're trying to render very large text,
    /// or using a *lot* of symbols.
    ///
    /// # What to do if the glyph cache is full
    ///
    /// Consider using alternative means of rendering large text, or
    /// increase your application's GPU capability requirements. The
    /// size of the glyph cache is limited by the OpenGL constant
    /// `GL_MAX_TEXTURE_SIZE`.
    pub fn is_glyph_cache_full(&self) -> bool {
        self.glyph_cache_filled
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
    /// ```ignore
    /// text_renderer.draw("Hello, World!", 10.0, 10.0, 0.0, 12.0)
    ///     .with_color((0.8, 0.5, 0.1, 1.0)) // Orange!
    ///     .with_cacheable(true) // Hello world never changes...
    ///     .finish();
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_text(
        &mut self,
        data: TextCacheable,
        z: f32,
        clip_area: Option<Rect>,
        color: (f32, f32, f32, f32),
        rotation: (f32, f32, f32),
        cacheable: bool,
        visible: bool,
    ) -> Option<Rect> {
        if data.text.is_empty() {
            return None;
        }

        let font_size = data.font_size;

        let draw_data_index = self.draw_datas.len();
        self.draw_datas.push(TextDrawData {
            position: (
                data.x as f32 / self.dpi_factor,
                data.y as f32 / self.dpi_factor,
            ),
            clip_area,
            color,
            rotation,
            font_size,
            z,
        });

        if let Some((glyphs, bounds)) = self.draw_cache.get(&data) {
            // Append the cached glyphs into the queue if they have been
            // cached, and stop here.
            if visible {
                crate::profiler::write(|p| p.layout_cache_hits += glyphs.len() as u32);
                self.glyphs.reserve(glyphs.len());
                for mut glyph in glyphs.iter().cloned() {
                    glyph.draw_data = draw_data_index;
                    self.glyphs.push(glyph);
                }
            }
            return *bounds;
        }

        let (mut min_x, mut min_y, mut max_x, mut max_y) = (
            std::f32::INFINITY,
            std::f32::INFINITY,
            std::f32::NEG_INFINITY,
            std::f32::NEG_INFINITY,
        );

        let max_line_width = data.max_line_width;
        let (x, y) = (data.x, data.y);
        let alignment = data.alignment;
        let text_glyphs: Vec<(char, GlyphId)> = data
            .text
            .chars()
            .map(|c| {
                let id = if let Some(id) = self.glyph_ids.get(&c) {
                    *id
                } else {
                    let id = self.font.get_glyph_id(c);
                    self.glyph_ids.insert(c, id);
                    id
                };
                (c, id)
            })
            .collect();

        let mut glyphs = Vec::with_capacity(text_glyphs.len());
        let mut cursor = Cursor::new(x, y);
        let mut i = 0;
        while i < text_glyphs.len() {
            let (line_stride, line_len) = if max_line_width.is_some()
                || alignment != Alignment::Left
            {
                let (line_stride, line_len, line_width) = get_line_length_and_width(
                    &mut *self.font,
                    cursor,
                    font_size,
                    max_line_width,
                    &text_glyphs[i..],
                );
                if let Some(max_line_width) = max_line_width {
                    cursor.x = get_line_start_x(cursor.x, line_width, max_line_width, alignment);
                }
                (line_stride, line_len)
            } else {
                get_line_length(&text_glyphs[i..])
            };

            let mut previous_glyph = None;
            for (_, glyph_id) in (&text_glyphs[i..]).iter().take(line_len) {
                let metrics = self.font.get_metric(*glyph_id, font_size);
                if let Some(prev) = previous_glyph {
                    let advance = self.font.get_advance(prev, *glyph_id, font_size);
                    cursor = cursor + advance;
                }

                let screen_location = metrics + cursor;
                min_x = min_x.min(screen_location.x as f32 / self.dpi_factor);
                min_y = min_y.min(screen_location.y as f32 / self.dpi_factor);
                max_x =
                    max_x.max((screen_location.x + screen_location.width) as f32 / self.dpi_factor);
                max_y = max_y
                    .max((screen_location.y + screen_location.height) as f32 / self.dpi_factor);
                glyphs.push(Glyph {
                    cursor,
                    metrics,
                    draw_data: draw_data_index,
                    id: *glyph_id,
                });
                previous_glyph = Some(*glyph_id);
            }

            i += line_stride;
            cursor.x = x;
            cursor = cursor + self.font.get_line_advance(font_size);
        }

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
        if visible {
            crate::profiler::write(|p| p.layout_cache_misses += len);
            self.glyphs.extend(&glyphs);
        }
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
        self.glyph_cache_filled = false;
        for glyph in &self.glyphs {
            let (base_x, base_y) = self.draw_datas[glyph.draw_data].position;
            let (radians, pivot_x, pivot_y) = self.draw_datas[glyph.draw_data].rotation;
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
                x: (glyph.cursor.x + glyph.metrics.x) as f32 - 0.5,
                y: (glyph.cursor.y + glyph.metrics.y) as f32 - 0.5,
                width: glyph.metrics.width as f32 + 1.0,
                height: glyph.metrics.height as f32 + 1.0,
            };

            // If the glyph is out of bounds, there's nothing to draw
            let in_window_bounds = |rect: Rect| {
                let (width, height) = self.window_size;
                rect.x + rect.width >= 0.0
                    && rect.y + rect.height >= 0.0
                    && rect.x < width
                    && rect.y < height
            };
            if !in_window_bounds(screen_location) {
                continue;
            }

            // If the clip area is out of bounds, there's nothing to draw either
            if let Some(area) = self.draw_datas[glyph.draw_data].clip_area {
                if !in_window_bounds(area) {
                    continue;
                }
            }

            match self
                .font
                .render_glyph(renderer, &mut self.cache, glyph.id, font_size)
            {
                Ok(texcoords) => {
                    let mut sprite = renderer
                        .draw(&self.call, z)
                        .with_physical_coordinates(screen_location)
                        .with_color(color);
                    if radians != 0.0 {
                        let dx = screen_location.x / self.dpi_factor - base_x;
                        let dy = screen_location.y / self.dpi_factor - base_y;
                        sprite = sprite.with_rotation(radians, pivot_x - dx, pivot_y - dy);
                    }
                    if let Some(area) = self.draw_datas[glyph.draw_data].clip_area {
                        sprite = sprite.with_clip_area(area);
                    }

                    let texcoords = Rect {
                        x: texcoords.x as f32 - 0.5,
                        y: texcoords.y as f32 - 0.5,
                        width: texcoords.width as f32 + 1.0,
                        height: texcoords.height as f32 + 1.0,
                    };
                    sprite.with_texture_coordinates(texcoords).finish();
                }
                Err(err) => match err {
                    GlyphNotRenderedError::GlyphCacheFull => self.glyph_cache_filled = true,
                },
            }
        }
    }

    /// Updates the dpi factor, resizes glyph cache if needed, clears
    /// up data from previous frame. Call at the beginning of a frame.
    pub fn prepare_new_frame(
        &mut self,
        renderer: &mut Renderer,
        dpi_factor: f32,
        window_width: f32,
        window_height: f32,
    ) {
        self.glyphs.clear();
        self.draw_datas.clear();
        self.cache.expire_one_step();
        self.cache.resize_if_needed(renderer);
        self.dpi_factor = dpi_factor;
        self.window_size = (window_width * dpi_factor, window_height * dpi_factor);
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
            .with_color((0.0, 0.0, 0.0, 1.0))
            .finish();
    }
}
