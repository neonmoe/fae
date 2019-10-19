use crate::{text::Alignment, DrawCallHandle, DrawCallParameters, Renderer};
use std::collections::HashMap;
use std::error::Error;

#[derive(Clone, Copy)]
struct RectPx(f32, f32, f32, f32);
#[derive(Clone, Copy)]
struct PositionPx(f32, f32);
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
            let glyph = Glyph {
                position: PositionPx(0.0, 0.0),
                size: c_metrics,
                z,
                draw_data: draw_data_index,
            };
            metrics.insert(c, glyph);
        }

        let line_height = font_size * 1.25; // Get from font

        self.glyphs.reserve(text.len());
        let mut cursor = PositionPx(x, y);
        let mut text_left = text;
        while !text_left.is_empty() {
            let (line_len, line_width) =
                get_line_length_and_width(&metrics, max_line_width, text_left);
            if let Some(max_line_width) = max_line_width {
                cursor.0 = get_line_start_x(cursor.0, line_width, max_line_width, alignment);
            }
            let mut previous_character = None;
            let mut chars_read = 0;
            for c in text_left.chars() {
                chars_read += 1;
                if chars_read >= line_len {
                    break;
                }
                if let Some(mut glyph) = metrics.get(&c).map(|m| m.clone()) {
                    glyph.position = cursor;
                    cursor.0 += get_char_width(&metrics, c, previous_character);
                    self.glyphs.push(glyph);
                }
                previous_character = Some(c);
            }
            text_left = move_forward_chars(text_left, line_len);
            cursor = PositionPx(x, cursor.1 + line_height);
        }
    }

    /// Makes the `draw_text` calls called before this function
    /// render. Should be called every frame before rendering.
    pub fn compose_draw_call(&mut self, renderer: &mut Renderer) {
        for glyph in &self.glyphs {
            let RectPx(x, y, w, h) = glyph.size;
            let PositionPx(x_, y_) = glyph.position;
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
    }
}

fn get_metrics(font_size: f32) -> RectPx {
    RectPx(0.0, 0.0, font_size / 2.0, font_size)
}

// TODO: Rip out the typesetting?

fn get_line_start_x(
    base_x: f32,
    line_width: f32,
    max_line_width: f32,
    alignment: Alignment,
) -> f32 {
    match alignment {
        Alignment::Left => base_x,
        Alignment::Center => base_x + (max_line_width - line_width) / 2.0,
        Alignment::Right => base_x + (max_line_width - line_width),
    }
}

fn move_forward_chars(s: &str, n: usize) -> &str {
    let mut chars = s.chars();
    for _ in 0..n {
        chars.next();
    }
    chars.as_str()
}

fn get_line_length_and_width(
    metrics: &HashMap<char, Glyph>,
    max_width: Option<f32>,
    mut s: &str,
) -> (usize, f32) {
    let mut len = 0;
    let mut total_width = 0.0;

    // Add words until the line can't fit more words
    'outer: while let Some((word_len, word_width)) =
        get_word_length(metrics, max_width.map(|w| w - total_width), &s, len == 0)
    {
        // Handle a word
        len += word_len;
        total_width += word_width;
        s = move_forward_chars(s, word_len);

        // Break if line is too long
        if s.is_empty() || max_width.iter().any(|w| total_width > *w) {
            break;
        }

        // Handle whitespace after word
        let len_before_whitespace = len;
        let mut previous_char = None;
        for c in s.chars() {
            len += 1;
            if c == '\n' {
                break 'outer;
            } else if c.is_whitespace() {
                // Update line width
                total_width += get_char_width(metrics, c, previous_char);
            } else {
                len -= 1;
                break;
            }
            previous_char = Some(c);
        }
        s = move_forward_chars(s, len - len_before_whitespace);

        // Break if line is too long
        if s.is_empty() || max_width.iter().any(|w| total_width > *w) {
            break;
        }
    }

    (len, total_width)
}

// Returns None if the word doesn't fit on this line (and might fit on
// the next), Some(word length, word width) if it does (or requires
// more than a whole line's worth of space, being the first word on
// the line).
fn get_word_length(
    metrics: &HashMap<char, Glyph>,
    max_width: Option<f32>,
    s: &str,
    starts_line: bool,
) -> Option<(usize, f32)> {
    let mut len = 0;
    let mut total_width = 0.0;
    let mut previous_char = None;
    for c in s.chars() {
        len += 1;
        if c.is_whitespace() {
            // Whitespace = end of word, cut off the whitespace and
            // return
            len -= 1;
            break;
        }

        // Check if line was overflowed
        let width = get_char_width(metrics, c, previous_char);
        total_width += width;
        if max_width.iter().any(|w| total_width > *w) {
            len -= 1;
            if starts_line {
                // This word is the first of the line, so we can't get
                // more space by starting the word on the next
                // line. So, just cut the word here and continue the
                // next line from where we now are.

                // TODO: Consider: text layout as a feature
                // Should we hyphenate, just cut off and go to the
                // next line, maybe even hyphenate before trying to go
                // to the next line (for justify-esque text)?
                break;
            } else {
                // This word is not the first on this line, so return
                // None to signal that the line ends here, to try and
                // fit this word on the next line.
                return None;
            }
        }

        previous_char = Some(c);
    }
    Some((len, total_width))
}

fn get_char_width(
    metrics: &HashMap<char, Glyph>,
    current_char: char,
    _previous_char: Option<char>,
) -> f32 {
    if let Some(current_glyph) = metrics.get(&current_char) {
        current_glyph.size.2 + 2.0 // + kerning between this char and the previous char
    } else {
        panic!(
            "'{}' has no metric but was in a string to be drawn",
            current_char
        )
    }
}
