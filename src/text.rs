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
    glyphs: Vec<PositionedGlyph<'a>>,
    z: f32,
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

pub(crate) fn queue_text(
    area: layout::Rect,
    z: f32,
    font_size: f32,
    text: &str,
    alignment: Alignment,
) {
    let mut cache = TEXT_CACHE.lock().unwrap();
    let rows = collect_glyphs(
        &mut cache,
        area.left,
        area.top,
        area.width(),
        font_size,
        text,
    );

    let mut final_glyphs = Vec::with_capacity(text.len());
    match alignment {
        Alignment::Left => {
            for row in rows {
                final_glyphs.extend_from_slice(&row);
            }
        }
        Alignment::Right => {
            for row in rows {
                if let Some((width, _)) = measure_text(&row) {
                    let offset = area.width() - width;
                    let row = offset_glyphs(row, offset, 0.0);
                    final_glyphs.extend_from_slice(&row);
                } else {
                    final_glyphs.extend_from_slice(&row);
                }
            }
        }
        Alignment::Center => {
            for row in rows {
                if let Some((width, _)) = measure_text(&row) {
                    let offset = (area.width() - width) / 2.0;
                    let row = offset_glyphs(row, offset, 0.0);
                    final_glyphs.extend_from_slice(&row);
                } else {
                    final_glyphs.extend_from_slice(&row);
                }
            }
        }
    }

    cache.cached_text.push(TextRender {
        glyphs: final_glyphs,
        z,
    });
}

fn measure_text<'a>(glyphs: &[PositionedGlyph<'a>]) -> Option<(f32, f32)> {
    let mut result: Option<layout::Rect> = None;
    let lock = DPI_SCALE.lock().unwrap();
    let dpi = *lock;

    for glyph in glyphs {
        if let Some(glyph_rect) = glyph.pixel_bounding_box() {
            if let Some(ref mut rect) = result {
                rect.left = rect.left.min(glyph_rect.min.x as f32 / dpi);
                rect.top = rect.top.min(glyph_rect.min.y as f32 / dpi);
                rect.right = rect.right.max(glyph_rect.max.x as f32 / dpi);
                rect.bottom = rect.bottom.max(glyph_rect.max.y as f32 / dpi);
            } else {
                result = Some(layout::Rect {
                    left: glyph_rect.min.x as f32 / dpi,
                    top: glyph_rect.min.y as f32 / dpi,
                    right: glyph_rect.max.x as f32 / dpi,
                    bottom: glyph_rect.max.y as f32 / dpi,
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

fn offset_glyphs<'a>(glyphs: Vec<PositionedGlyph<'a>>, x: f32, y: f32) -> Vec<PositionedGlyph<'a>> {
    glyphs
        .into_iter()
        .map(|glyph| {
            let position = glyph.position() + vector(x, y);
            glyph.into_unpositioned().positioned(position)
        })
        .collect()
}

fn collect_glyphs<'a>(
    cache: &mut TextCache<'a>,
    x: f32,
    y: f32,
    width: f32,
    font_size: f32,
    text: &str,
) -> Vec<Vec<PositionedGlyph<'a>>> {
    let dpi;
    let scale;
    {
        let lock = DPI_SCALE.lock().unwrap();
        dpi = *lock;
        scale = Scale::uniform(font_size * dpi)
    }
    let x = x * dpi;
    let y = y * dpi;

    let mut rows = Vec::new();
    rows.push(Vec::with_capacity(text.len()));
    if let Some(ref font) = &cache.font {
        let v_metrics = font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = point(x, y + v_metrics.ascent);
        let mut last_glyph_id = None;

        let next_row = |caret: &mut Point<f32>, rows: &mut Vec<Vec<_>>| {
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
                if c == '\n' {
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
            if caret.x > x + width {
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
            caret.x += glyph.unpositioned().h_metrics().advance_width;

            rows.last_mut().unwrap().push(glyph);
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
                cache.queue_glyph(0, glyph.clone());
            }
        }

        unsafe {
            let tex = renderer::get_texture(1);
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

        let lock = DPI_SCALE.lock().unwrap();
        let dpi = *lock;

        for text in &text_cache.cached_text {
            let z = text.z;
            for glyph in &text.glyphs {
                if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, &glyph) {
                    let coords = (
                        screen_rect.min.x as f32 / dpi,
                        screen_rect.min.y as f32 / dpi,
                        screen_rect.max.x as f32 / dpi,
                        screen_rect.max.y as f32 / dpi,
                    );
                    let texcoords = (uv_rect.min.x, uv_rect.min.y, uv_rect.max.x, uv_rect.max.y);
                    renderer::draw_quad(coords, texcoords, z, 1);
                }
            }
        }
    }

    text_cache.cached_text.clear();
}
