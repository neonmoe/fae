//! The text rendering module.

mod fonts;
#[cfg(feature = "font8x8" /* or font-kit, in the future */)]
mod glyph_cache;
mod layout;
pub(crate) mod types;

pub use crate::text::types::Alignment;

#[cfg(feature = "font8x8" /* or font-kit, in the future */)]
use crate::text::glyph_cache::*;
use crate::text::layout::*;
use crate::text::types::*;
use crate::{DrawCallHandle, Renderer};
use std::collections::HashMap;

/// Holds the state required for text rendering, such as the font, and
/// a text draw call queue.
pub struct TextRenderer {
    call: DrawCallHandle,
    glyphs: Vec<Glyph>,
    draw_datas: Vec<TextDrawData>,
    dpi_factor: f32,
    font: Box<dyn FontProvider>,
}

impl TextRenderer {
    /// Creates a new text renderer without any external fonts.
    ///
    /// `smoothed` controls the `magnification_smoothing` value of the
    /// underlying draw call.
    ///
    /// If the `font8x8` feature is enabled, will use those
    /// glyphs. Otherwise, will draw squares in the place of those
    /// glyphs.
    pub fn create_simple(renderer: &mut Renderer, smoothed: bool) -> TextRenderer {
        #[cfg(feature = "font8x8" /* or font-kit, in the future */)]
        let (glyph_cache, call) =
            GlyphCache::create_cache_and_draw_call(renderer, 128, 128, smoothed);

        #[cfg(not(feature = "font8x8" /* or font-kit, in the future */))]
        let call = renderer.create_draw_call(crate::renderer::DrawCallParameters {
            alpha_blending: false,
            ..Default::default()
        });

        TextRenderer {
            call,
            glyphs: Vec::new(),
            draw_datas: Vec::new(),
            dpi_factor: 1.0,
            font: {
                #[cfg(not(feature = "font8x8"))]
                let provider = fonts::DummyProvider;
                #[cfg(feature = "font8x8")]
                let provider = fonts::Font8x8Provider::new(glyph_cache);
                Box::new(provider)
            },
        }
    }

    /// Updates the DPI factor that will be taken into account during
    /// text rendering. If the window DPI changes, this should be
    /// called with the new factor before new text draw calls.
    // TODO: Refactor this function out, it's not good
    pub fn update_dpi_factor(&mut self, dpi_factor: f32) {
        self.dpi_factor = dpi_factor;
    }

    /// Draws text, and returns a bounding box `(min_x, min_y, max_x,
    /// max_y)` of all glyphs drawn, if any were.
    ///
    /// - `text`: The rendered text.
    /// - `(x, y, z)`: The position (top-left) of the rendered text
    /// area. TODO: The y should be the baseline of the text
    /// - `font_size`: The size of the font.
    /// - `max_line_width`: The width at which the text will wrap. An
    /// effort is made to break lines at word boundaries.
    /// - `clip_area`: The area which defines where the text will be
    /// rendered. Text outside the area will be cut off. For an
    /// example use case, think editable text boxes: the clip area
    /// would be the text box's inner are.
    pub fn draw_text(
        &mut self,
        text: &str,
        (x, y, z): (f32, f32, f32),
        font_size: f32,
        alignment: Alignment,
        color: (f32, f32, f32, f32),
        max_line_width: Option<f32>,
        clip_area: Option<(f32, f32, f32, f32)>,
    ) -> Option<(f32, f32, f32, f32)> {
        if text.len() == 0 {
            return None;
        }

        let draw_data_index = self.draw_datas.len();
        self.draw_datas.push(TextDrawData {
            clip_area,
            color,
            font_size,
            z,
        });

        let mut metrics = HashMap::new();
        for c in text.chars() {
            if metrics.get(&c).is_some() {
                continue;
            }

            let glyph_id = self.font.get_glyph_id(c);
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

        self.glyphs.reserve(text.len());
        let line_height = self.font.get_line_height(font_size);
        let mut cursor = PositionPx { x, y };
        let mut text_left = text;
        while !text_left.is_empty() {
            let (line_len, line_width) = get_line_length_and_width(
                &self.font,
                &metrics,
                font_size,
                max_line_width,
                text_left,
            );
            if let Some(max_line_width) = max_line_width {
                cursor.x = get_line_start_x(cursor.x, line_width, max_line_width, alignment);
            }

            let mut previous_character = None;
            let mut chars_read = 0;
            let mut chars = text_left.chars();
            for c in &mut chars {
                chars_read += 1;
                if let Some(metric) = metrics.get(&c).map(|m| m.clone()) {
                    // Advance the cursor, if this is not the first character
                    if let Some(previous_character) = previous_character {
                        if let Some(advance) =
                            get_char_advance(&self.font, &metrics, font_size, c, previous_character)
                        {
                            cursor.x += advance;
                        }
                    }

                    let screen_location = metric.size + cursor;
                    min_x = min_x.min(screen_location.x);
                    min_y = min_y.min(screen_location.y);
                    max_x = max_x.max(screen_location.x + screen_location.w);
                    max_y = max_y.max(screen_location.y + screen_location.h);
                    self.glyphs.push(Glyph {
                        screen_location,
                        draw_data: draw_data_index,
                        id: metric.glyph_id,
                    });
                }
                previous_character = Some(c);
                if chars_read >= line_len {
                    break;
                }
            }
            text_left = chars.as_str();
            cursor = PositionPx {
                x,
                y: cursor.y + line_height,
            };
        }

        if let Some((clip_min_x, clip_min_y, clip_max_x, clip_max_y)) = clip_area {
            min_x = min_x.max(clip_min_x);
            min_y = min_y.max(clip_min_y);
            max_x = max_x.min(clip_max_x);
            max_y = max_y.min(clip_max_y);
        }

        if min_x == std::f32::INFINITY
            || min_y == std::f32::INFINITY
            || max_x == std::f32::NEG_INFINITY
            || max_y == std::f32::NEG_INFINITY
        {
            None
        } else {
            Some((min_x, min_y, max_x, max_y))
        }
    }

    /// Makes the `draw_text` calls called before this function
    /// render. Should be called every frame before rendering.
    pub fn compose_draw_call(&mut self, renderer: &mut Renderer) {
        crate::profiler::insert_profiling_data("glyphs drawn", "0");
        crate::profiler::insert_profiling_data("glyphs rendered", "0");

        for glyph in &self.glyphs {
            let font_size = self.draw_datas[glyph.draw_data].font_size;
            let color = self.draw_datas[glyph.draw_data].color;
            let z = self.draw_datas[glyph.draw_data].z;

            let RectPx { x, y, w, h } = glyph.screen_location;
            let position = (x, y, x + w, y + h);
            let RectUv { x, y, w, h } = match self.font.render_glyph(glyph.id, font_size) {
                Some(uvs) => uvs,
                None => continue,
            };
            let texcoords = (x, y, x + w, y + h);

            if let Some(clip_area) = self.draw_datas[glyph.draw_data].clip_area {
                renderer.draw_quad_clipped(
                    clip_area,
                    position,
                    texcoords,
                    color,
                    (0.0, 0.0, 0.0),
                    z,
                    &self.call,
                );
            } else {
                renderer.draw_quad(position, texcoords, color, (0.0, 0.0, 0.0), z, &self.call);
            }
        }
        self.glyphs.clear();
        self.draw_datas.clear();
        self.font.update_glyph_cache_expiration();
    }

    /// Draws the glyph cache texture in the given screen-space quad,
    /// for debugging.
    pub fn debug_draw_glyph_cache(
        &self,
        renderer: &mut Renderer,
        quad: (f32, f32, f32, f32),
        z: f32,
    ) {
        renderer.draw_quad(
            quad,
            (0.0, 0.0, 1.0, 1.0),
            (0.0, 0.0, 0.0, 1.0),
            (0.0, 0.0, 0.0),
            z,
            &self.call,
        );
    }
}
