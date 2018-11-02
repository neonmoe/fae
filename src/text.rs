use gl;
use gl::types::*;
use renderer;
use rusttype::gpu_cache::Cache;
use rusttype::*;
use std::cell::RefCell;
use std::error::Error;
use std::sync::Mutex;
use ui::layout;
use unicode_normalization::UnicodeNormalization;

lazy_static! {
    static ref TEXT_CACHE: Mutex<TextCache<'static>> = Mutex::new(TextCache {
        font: None,
        cache: None,
        cached_text: Vec::new(),
    });
    static ref DPI_SCALE: Mutex<f32> = Mutex::new(1.0);
}

pub(crate) const GLYPH_CACHE_WIDTH: u32 = 1024;
pub(crate) const GLYPH_CACHE_HEIGHT: u32 = 1024;

struct TextCache<'a> {
    font: Option<Font<'a>>,
    cache: Option<RefCell<Cache<'a>>>,
    cached_text: Vec<TextRender<'a>>,
}

struct TextRender<'a> {
    glyphs: Vec<SizedGlyph<'a>>,
    clip_area: layout::Rect,
    z: f32,
}

#[derive(Clone)]
struct SizedGlyph<'a> {
    glyph: PositionedGlyph<'a>,
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
#[derive(Clone, Copy, Debug)]
pub enum Alignment {
    /// Text is aligned to the left.
    Left,
    /// Text is aligned to the right.
    Right,
    /// Text is centered.
    Center,
}

/// Initialize fonts. Handled by `window_bootstrap`. This must be done
/// before any drawing calls.
///
/// `font_data` should a Vec of the bytes of a .ttf file. To load the
/// image at compile-time, you could run the following (of course,
/// with your own path):
/// ```
/// fungui::initialize_font(include_bytes!("resources/FiraSans.ttf").to_vec());
/// ```
pub fn initialize_font(font_data: Vec<u8>) -> Result<(), Box<Error>> {
    let cache = TextCache {
        font: Some(Font::from_bytes(font_data)?),
        cache: Some(RefCell::new(
            Cache::builder()
                .dimensions(GLYPH_CACHE_WIDTH, GLYPH_CACHE_HEIGHT)
                .build(),
        )),
        cached_text: Vec::new(),
    };
    let mut lock = TEXT_CACHE.lock()?;
    *lock = cache;
    Ok(())
}

pub(crate) fn update_dpi(dpi: f32) {
    let mut lock = DPI_SCALE.lock().unwrap();
    *lock = dpi;
}

#[inline]
fn get_dpi() -> f32 {
    let lock = DPI_SCALE.lock().unwrap();
    *lock
}

pub(crate) fn queue_text(
    text: &str,
    (x, y, z): (f32, f32, f32),
    width: f32,
    font_size: f32,
    alignment: Alignment,
    multiline: bool,
    cursor: Option<&mut TextCursor>,
) {
    let mut cache = TEXT_CACHE.lock().unwrap();
    let rows = collect_glyphs(&mut cache, x, y, width, multiline, font_size, text);

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
                let row = if let Some((row_width, _)) = measure_text(&row) {
                    let offset = width - row_width;
                    offset_glyphs(row, offset, 0.0)
                } else {
                    row
                };
                final_glyphs.extend_from_slice(&row);
            }
        }

        Alignment::Center => {
            for row in rows {
                let row = if let Some((row_width, _)) = measure_text(&row) {
                    let offset = (width - row_width) / 2.0;
                    offset_glyphs(row, offset, 0.0)
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
            let cursor_rect = measure_text_at_index(&final_glyphs, index).unwrap();
            cursor_rect.left
        } else if index > 0 {
            let cursor_rect = measure_text_at_index(&final_glyphs, index - 1).unwrap();
            cursor_rect.right
        } else if let Some(rect) = measure_text_at_index(&final_glyphs, 0) {
            rect.left
        } else {
            match alignment {
                Alignment::Left => x,
                Alignment::Right => x + width - 4.0 * get_dpi(),
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
            final_glyphs = offset_glyphs(final_glyphs, offset_x, 0.0);
        }
    }

    cache.cached_text.push(TextRender {
        glyphs: final_glyphs,
        clip_area: layout::Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + font_size,
        },
        z,
    });
}

/// Will only return `None` when `index >= glyphs.len()`.
fn measure_text_at_index<'a>(glyphs: &[SizedGlyph<'a>], index: usize) -> Option<layout::Rect> {
    if index >= glyphs.len() {
        return None;
    }

    let dpi = get_dpi();

    let width = glyphs[index].width;
    let glyph = &glyphs[index].glyph;
    let position = glyph.position();
    if let Some(rect) = glyph.pixel_bounding_box() {
        return Some(layout::Rect {
            left: rect.min.x as f32 / dpi,
            top: rect.min.y as f32 / dpi,
            right: rect.max.x as f32 / dpi,
            bottom: rect.max.y as f32 / dpi,
        });
    } else {
        return Some(layout::Rect {
            left: position.x / dpi,
            top: position.y / dpi,
            right: (position.x + width) / dpi,
            bottom: position.y / dpi,
        });
    }
}

fn measure_text<'a>(glyphs: &[SizedGlyph<'a>]) -> Option<(f32, f32)> {
    let mut result: Option<layout::Rect> = None;

    for i in 0..glyphs.len() {
        if let Some(glyph_rect) = measure_text_at_index(glyphs, i) {
            if let Some(ref mut rect) = result {
                rect.left = rect.left.min(glyph_rect.left);
                rect.top = rect.top.min(glyph_rect.top);
                rect.right = rect.right.max(glyph_rect.right);
                rect.bottom = rect.bottom.max(glyph_rect.bottom);
            } else {
                result = Some(layout::Rect {
                    left: glyph_rect.left,
                    top: glyph_rect.top,
                    right: glyph_rect.right,
                    bottom: glyph_rect.bottom,
                });
            }
        }
    }

    if let Some(rect) = result {
        Some((rect.right - rect.left, rect.bottom - rect.top))
    } else {
        None
    }
}

fn offset_glyphs(glyphs: Vec<SizedGlyph>, x: f32, y: f32) -> Vec<SizedGlyph> {
    let dpi = get_dpi();
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

fn collect_glyphs<'a>(
    cache: &mut TextCache<'a>,
    x: f32,
    y: f32,
    width: f32,
    multiline: bool,
    font_size: f32,
    text: &str,
) -> Vec<Vec<SizedGlyph<'a>>> {
    let dpi = get_dpi();
    let scale = Scale::uniform(font_size * dpi);
    let x = x * dpi;
    let y = y * dpi;

    let mut rows = Vec::new();
    rows.push(Vec::with_capacity(text.len()));
    if let Some(ref font) = &cache.font {
        let v_metrics = font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = point(x, y + v_metrics.ascent);
        let mut last_glyph_id = None;

        let next_row = |caret: &mut Point<f32>, rows: &mut Vec<Vec<SizedGlyph<'a>>>| {
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

            let glyph = font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += font.pair_kerning(scale, id, glyph.id());
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
    }
    rows
}

pub(crate) fn draw_text() {
    let mut text_cache = TEXT_CACHE.lock().unwrap();

    if let Some(ref cache) = &text_cache.cache {
        let mut cache = cache.borrow_mut();

        for text in &text_cache.cached_text {
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

        let dpi = get_dpi();

        for text in &text_cache.cached_text {
            let z = text.z;
            let clip_area = text.clip_area.to_tuple();
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
    }

    text_cache.cached_text.clear();
}
