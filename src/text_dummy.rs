use crate::{text::Alignment, text_layout::*, DrawCallHandle, DrawCallParameters, Renderer};
use std::collections::HashMap;
use std::error::Error;

#[derive(Clone, Copy)]
pub(crate) struct RectPx {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
#[derive(Clone, Copy)]
pub(crate) struct PositionPx {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Metric {
    pub(crate) size: RectPx,
}
#[derive(Clone, Copy)]
struct Glyph {
    position: PositionPx,
    size: RectPx,
    z: f32,
    draw_data: usize,
}

struct TextDrawData {
    clip_area: Option<(f32, f32, f32, f32)>,
    color: (f32, f32, f32, f32),
}

/// Holds the state required for text rendering, such as the font, and
/// a text draw call queue.
pub struct TextRenderer {
    call: DrawCallHandle,
    glyphs: Vec<Glyph>,
    draw_datas: Vec<TextDrawData>,
    dpi_factor: f32,
}

impl TextRenderer {
    /// Creates a new text renderer.
    ///
    /// - `font_data`: The bytes that consist a .ttf file. See the `rusttype` crate's documentation for what kinds of fonts are supported.
    ///
    /// - `subpixel_accurate`: If true, glyphs will be rendered if
    /// their subpixel position differs by very small amounts, to
    /// render the font more accurately for that position. In
    /// practice, I haven't seen any difference, so I'd recommend
    /// setting this to false. (Internally this maps to `rusttype`'s
    /// `CacheBuilder`'s position tolerance value, true = 0.1, false =
    /// 1.0).
    pub fn create(
        _font_data: Vec<u8>,
        _subpixel_accurate: bool,
        renderer: &mut Renderer,
    ) -> Result<TextRenderer, Box<dyn Error>> {
        Ok(TextRenderer {
            call: renderer.create_draw_call(DrawCallParameters {
                alpha_blending: false,
                ..Default::default()
            }),
            glyphs: Vec::new(),
            draw_datas: Vec::new(),
            dpi_factor: 1.0,
        })
    }

    /// Updates the DPI factor that will be taken into account during
    /// text rendering. If the window DPI changes, this should be
    /// called with the new factor before new text draw calls.
    // TODO: Refactor this function out, it's not good
    pub fn update_dpi_factor(&mut self, dpi_factor: f32) {
        self.dpi_factor = dpi_factor;
    }

    /// Draws text.
    ///
    /// - `text`: The rendered text.
    /// - `(x, y, z)`: The position (top-left) of the rendered text
    /// area.
    /// - `font_size`: The size of the font.
    /// - `max_line_width`: The width at which the text will wrap. An
    /// effort is made to break lines at word boundaries.
    /// - `clip_area`: The area which defines where the text will be
    /// rendered. Text outside the area will be cut off. For an
    /// example use case, think editable text boxes: the clip area
    /// would be the text box.
    pub fn draw_text(
        &mut self,
        text: &str,
        (x, y, z): (f32, f32, f32),
        font_size: f32,
        alignment: Alignment,
        color: (f32, f32, f32, f32),
        max_line_width: Option<f32>,
        clip_area: Option<(f32, f32, f32, f32)>,
    ) {
        let draw_data_index = self.draw_datas.len();
        self.draw_datas.push(TextDrawData { clip_area, color });

        let mut metrics = HashMap::new();
        for c in text.chars() {
            if metrics.get(&c).is_some() {
                continue;
            }

            let c_metrics = get_metrics(font_size);
            let glyph = Metric { size: c_metrics };
            metrics.insert(c, glyph);
        }

        let line_height = font_size * 1.25; // Get from font

        self.glyphs.reserve(text.len());
        let mut cursor = PositionPx { x, y };
        let mut text_left = text;
        while !text_left.is_empty() {
            let (line_len, line_width) =
                get_line_length_and_width(&metrics, max_line_width, text_left);
            if let Some(max_line_width) = max_line_width {
                cursor.x = get_line_start_x(cursor.x, line_width, max_line_width, alignment);
            }
            let mut previous_character = None;
            let mut chars_read = 0;
            let mut chars = text_left.chars();
            for c in &mut chars {
                chars_read += 1;
                if chars_read >= line_len {
                    break;
                }
                if let Some(metric) = metrics.get(&c).map(|m| m.clone()) {
                    self.glyphs.push(Glyph {
                        position: cursor,
                        size: metric.size,
                        z,
                        draw_data: draw_data_index,
                    });
                    cursor.x += get_char_width(&metrics, c, previous_character);
                }
                previous_character = Some(c);
            }
            text_left = chars.as_str();
            cursor = PositionPx {
                x,
                y: cursor.y + line_height,
            };
        }
    }

    /// Makes the `draw_text` calls called before this function
    /// render. Should be called every frame before rendering.
    pub fn compose_draw_call(&mut self, renderer: &mut Renderer) {
        for glyph in &self.glyphs {
            let RectPx { x, y, w, h } = glyph.size;
            let PositionPx { x: x_, y: y_ } = glyph.position;
            let (x0, y0) = (x + x_, y + y_);
            let (x1, y1) = (x0 + w, y0 + h);
            let position = (x0, y0, x1, y1);
            let inner_position = (
                x0 + 1.0 / self.dpi_factor,
                y0 + 1.0 / self.dpi_factor,
                x1 - 1.0 / self.dpi_factor,
                y1 - 1.0 / self.dpi_factor,
            );

            let color = self.draw_datas[glyph.draw_data].color;
            if let Some(clip_area) = self.draw_datas[glyph.draw_data].clip_area {
                renderer.draw_quad_clipped(
                    clip_area,
                    position,
                    (-1.0, -1.0, -1.0, -1.0),
                    color,
                    (0.0, 0.0, 0.0),
                    glyph.z,
                    &self.call,
                );
            } else {
                // Draw the inner part first: alpha blending is off in
                // the dummy renderer, so pixels won't get overdrawn
                // (this gets drawn first, and because the z is the
                // same, the next draw call will not be drawn over
                // this)
                renderer.draw_quad(
                    inner_position,
                    (-1.0, -1.0, -1.0, -1.0),
                    (1.0 - color.0, 1.0 - color.1, 1.0 - color.2, color.3),
                    (0.0, 0.0, 0.0),
                    glyph.z,
                    &self.call,
                );
                renderer.draw_quad(
                    position,
                    (-1.0, -1.0, -1.0, -1.0),
                    color,
                    (0.0, 0.0, 0.0),
                    glyph.z,
                    &self.call,
                );
            }
        }
        self.glyphs.clear();
        self.draw_datas.clear();
    }
}

fn get_metrics(font_size: f32) -> RectPx {
    RectPx {
        x: 0.0,
        y: 0.0,
        w: font_size / 2.0,
        h: font_size,
    }
}
