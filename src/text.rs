use gl;
use gl::types::*;
use rect;
use renderer;
use rusttype::gpu_cache::Cache;
use rusttype::*;
use std::cell::RefCell;
use std::error::Error;
use unicode_normalization::UnicodeNormalization;

pub(crate) const GLYPH_CACHE_WIDTH: u32 = 1024;
pub(crate) const GLYPH_CACHE_HEIGHT: u32 = 1024;

struct TextRender {
    glyphs: Vec<SizedGlyph>,
    clip_area: rect::Rect,
    z: f32,
}

#[derive(Clone)]
struct SizedGlyph {
    glyph: PositionedGlyph<'static>,
    width: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct TextCursor {
    pub index: usize,
    pub blink_visibility: bool,
    offset_min: Option<f32>,
    offset_max: Option<f32>,
}

impl TextCursor {
    pub fn new(index: usize, blink_visibility: bool) -> TextCursor {
        TextCursor {
            index,
            blink_visibility,
            offset_min: None,
            offset_max: None,
        }
    }
}

/// Defines the alignment of text.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Alignment {
    /// Text is aligned to the left.
    Left,
    /// Text is aligned to the right.
    Right,
    /// Text is centered.
    Center,
}

/// Will only return `None` when `index >= glyphs.len()`.
fn measure_text_at_index(glyphs: &[SizedGlyph], index: usize, dpi: f32) -> Option<rect::Rect> {
    if index >= glyphs.len() {
        return None;
    }

    let width = glyphs[index].width;
    let glyph = &glyphs[index].glyph;
    let position = glyph.position();
    if let Some(rect) = glyph.pixel_bounding_box() {
        return Some(rect::Rect::Coords(
            rect.min.x as f32 / dpi,
            rect.min.y as f32 / dpi,
            rect.max.x as f32 / dpi,
            rect.max.y as f32 / dpi,
        ));
    } else {
        return Some(rect::Rect::Dims(
            position.x / dpi,
            position.y / dpi,
            width / dpi,
            1.0,
        ));
    }
}

fn measure_text(glyphs: &[SizedGlyph], dpi: f32) -> Option<(f32, f32)> {
    let mut result: Option<rect::Rect> = None;

    for i in 0..glyphs.len() {
        if let Some(glyph_rect) = measure_text_at_index(glyphs, i, dpi) {
            if let Some(ref mut rect) = result {
                let (x0, y0, x1, y1) = (
                    rect.left().min(glyph_rect.left()),
                    rect.top().min(glyph_rect.top()),
                    rect.right().max(glyph_rect.right()),
                    rect.bottom().max(glyph_rect.bottom()),
                );
                rect.set_coords(x0, y0, x1, y1);
            } else {
                result = Some(glyph_rect);
            }
        }
    }

    if let Some(rect) = result {
        Some((rect.width(), rect.height()))
    } else {
        None
    }
}

fn offset_glyphs(glyphs: Vec<SizedGlyph>, x: f32, y: f32, dpi: f32) -> Vec<SizedGlyph> {
    glyphs
        .into_iter()
        .map(|glyph| offset_glyph(glyph, x, y, dpi))
        .collect()
}

fn offset_glyph(glyph: SizedGlyph, x: f32, y: f32, dpi: f32) -> SizedGlyph {
    let width = glyph.width;
    let glyph = glyph.glyph;
    let position = glyph.position() + vector(x, y) * dpi;
    SizedGlyph {
        width,
        glyph: glyph.into_unpositioned().positioned(position),
    }
}

pub struct TextRenderer {
    font: Font<'static>,
    cache: RefCell<Cache<'static>>,
    cached_text: Vec<TextRender>,
    dpi: f32,
}

impl TextRenderer {
    pub(crate) fn create(font_data: Vec<u8>) -> Result<TextRenderer, Box<Error>> {
        Ok(TextRenderer {
            font: Font::from_bytes(font_data)?,
            cache: RefCell::new(
                Cache::builder()
                    .dimensions(GLYPH_CACHE_WIDTH, GLYPH_CACHE_HEIGHT)
                    .build(),
            ),
            cached_text: Vec::new(),
            dpi: 1.0,
        })
    }

    pub(crate) fn update_dpi(&mut self, dpi: f32) {
        self.dpi = dpi;
    }

    pub(crate) fn queue_text(
        &mut self,
        text: &str,
        (x, y, z): (f32, f32, f32),
        width: f32,
        font_size: f32,
        alignment: Alignment,
        multiline: bool,
        cursor: Option<&mut TextCursor>,
    ) {
        let rows = self.collect_glyphs(x, y, width, multiline, font_size, text);
        let dpi = self.dpi;

        let mut final_glyphs = Vec::with_capacity(text.len());

        // Collect the rows and offset them according to the alignment
        match alignment {
            Alignment::Left => {
                for row in rows {
                    final_glyphs.extend_from_slice(&row);
                }
            }

            Alignment::Right => {
                for row in rows {
                    let row = if let Some((row_width, _)) = measure_text(&row, dpi) {
                        let offset = width - row_width;
                        offset_glyphs(row, offset, 0.0, dpi)
                    } else {
                        row
                    };
                    final_glyphs.extend_from_slice(&row);
                }
            }

            Alignment::Center => {
                for row in rows {
                    let row = if let Some((row_width, _)) = measure_text(&row, dpi) {
                        let offset = (width - row_width) / 2.0;
                        offset_glyphs(row, offset, 0.0, dpi)
                    } else {
                        row
                    };
                    final_glyphs.extend_from_slice(&row);
                }
            }
        }

        // Add cursor character
        if let Some(&mut TextCursor {
            index,
            blink_visibility,
            offset_min,
            offset_max,
        }) = cursor
        {
            let mut cursor_x = if index > 0 && index < final_glyphs.len() {
                let cursor_rect = measure_text_at_index(&final_glyphs, index, dpi).unwrap();
                cursor_rect.left()
            } else if index > 0 {
                let cursor_rect = measure_text_at_index(&final_glyphs, index - 1, dpi).unwrap();
                cursor_rect.right()
            } else if let Some(rect) = measure_text_at_index(&final_glyphs, 0, dpi) {
                rect.left()
            } else {
                match alignment {
                    Alignment::Left => x,
                    Alignment::Right => x + width - 4.0 * dpi,
                    Alignment::Center => x + width / 2.0,
                }
            };

            // TODO: Clean up offsets caused by cursors
            // Because this code is a *mess*.

            let cursor = cursor.unwrap();
            let mut current_offset_x = None;

            if let Some(offset_min) = offset_min {
                let new_x = cursor_x + offset_min;
                if new_x >= x && new_x < x + width {
                    current_offset_x = Some(offset_min);
                } else {
                    cursor.offset_min = None;
                    if new_x >= x + width {
                        current_offset_x = Some(x + width - cursor_x);
                        cursor.offset_max = current_offset_x;
                    } else {
                        current_offset_x = Some(x - cursor_x);
                        cursor.offset_min = current_offset_x;
                    }
                }
            } else if let Some(offset_max) = offset_max {
                let new_x = cursor_x + offset_max;
                if new_x >= x && new_x < x + width {
                    current_offset_x = Some(offset_max);
                } else {
                    cursor.offset_max = None;
                    if new_x < x {
                        current_offset_x = Some(x - cursor_x);
                        cursor.offset_min = current_offset_x;
                    } else {
                        current_offset_x = Some(x + width - cursor_x);
                        cursor.offset_max = current_offset_x;
                    }
                }
            }

            if current_offset_x.is_none() {
                if cursor_x >= x + width {
                    let offset = x + width - cursor_x;
                    cursor.offset_min = None;
                    cursor.offset_max = Some(offset);
                    current_offset_x = Some(offset);
                } else if cursor_x < x {
                    let offset = x - cursor_x;
                    cursor.offset_min = Some(offset);
                    cursor.offset_max = None;
                    current_offset_x = Some(offset);
                } else {
                    cursor.offset_min = None;
                    cursor.offset_max = None;
                }
            }

            if blink_visibility {
                let x = current_offset_x.unwrap_or(0.0) + cursor_x;
                renderer::draw_colored_quad(
                    (x - 0.5, y + 1.0, x + 0.5, y + font_size - 1.0),
                    (0, 0, 0, 0xFF),
                    z,
                    renderer::DRAW_CALL_INDEX_UI,
                );
            }

            if let Some(offset_x) = current_offset_x {
                final_glyphs = offset_glyphs(final_glyphs, offset_x, 0.0, dpi);
            }
        }

        self.cached_text.push(TextRender {
            glyphs: final_glyphs,
            clip_area: rect::Rect::Dims(x, y, width, font_size),
            z,
        });
    }

    fn collect_glyphs(
        &self,
        x: f32,
        y: f32,
        width: f32,
        multiline: bool,
        font_size: f32,
        text: &str,
    ) -> Vec<Vec<SizedGlyph>> {
        let dpi = self.dpi;
        let scale = Scale::uniform(font_size * dpi);
        let x = x * dpi;
        let y = y * dpi;

        let mut rows = Vec::new();
        rows.push(Vec::with_capacity(text.len()));
        let v_metrics = self.font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = point(x, y + v_metrics.ascent);
        let mut last_glyph_id = None;

        let next_row = |caret: &mut Point<f32>, rows: &mut Vec<Vec<SizedGlyph>>| {
            *caret = point(x, caret.y + advance_height);
            // Pre-allocate based on the last row's length
            let len = rows.last().unwrap().len();
            rows.push(Vec::with_capacity(len));
        };

        let chars: Vec<char> = text.nfc().collect();
        let mut i = 0;
        let mut current_word_length = 0;
        while i < chars.len() {
            let c = chars[i];
            i += 1;
            if c.is_control() {
                if c == '\n' && multiline {
                    next_row(&mut caret, &mut rows);
                }
                continue;
            }
            if c == ' ' {
                current_word_length = 0;
            } else {
                current_word_length += 1;
            }

            let glyph = self.font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += self.font.pair_kerning(scale, id, glyph.id());
            }

            if caret.x > x + width && multiline {
                if let Some(ref mut last_row) = rows.last_mut() {
                    let len = last_row.len();
                    if current_word_length < len {
                        last_row.truncate(len - current_word_length);
                        i -= current_word_length;
                    } else {
                        i -= 1;
                    }
                    current_word_length = 0;
                }
                next_row(&mut caret, &mut rows);
                continue;
            } else {
                last_glyph_id = Some(glyph.id());
            }

            let glyph = glyph.scaled(scale).positioned(caret);
            let advance_width = glyph.unpositioned().h_metrics().advance_width;
            caret.x += advance_width;

            rows.last_mut().unwrap().push(SizedGlyph {
                glyph,
                width: advance_width,
            });
        }
        rows
    }

    pub(crate) fn draw_text(&mut self) {
        let dpi = self.dpi;
        let mut cache = self.cache.borrow_mut();

        for text in &self.cached_text {
            for glyph in &text.glyphs {
                cache.queue_glyph(0, glyph.glyph.clone());
            }
        }

        let tex = renderer::get_texture(renderer::DRAW_CALL_INDEX_TEXT);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }

        let upload_new_texture = |rect: Rect<u32>, data: &[u8]| unsafe {
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                rect.min.x as GLint,
                rect.min.y as GLint,
                rect.width() as GLint,
                rect.height() as GLint,
                gl::RED as GLuint,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
        };
        cache.cache_queued(upload_new_texture).ok();

        for text in &self.cached_text {
            let z = text.z;
            let clip_area = text.clip_area.coords();
            for glyph in &text.glyphs {
                if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, &glyph.glyph) {
                    let coords = (
                        screen_rect.min.x as f32 / dpi,
                        screen_rect.min.y as f32 / dpi,
                        screen_rect.max.x as f32 / dpi,
                        screen_rect.max.y as f32 / dpi,
                    );
                    let texcoords = (uv_rect.min.x, uv_rect.min.y, uv_rect.max.x, uv_rect.max.y);
                    renderer::draw_quad_clipped(
                        coords,
                        texcoords,
                        (0, 0, 0, 0xFF),
                        clip_area,
                        z,
                        renderer::DRAW_CALL_INDEX_TEXT,
                    );
                }
            }
        }

        self.cached_text.clear();
    }
}
