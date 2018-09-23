use super::*;
use gl;
use rusttype::gpu_cache::Cache;
use rusttype::*;
use std::cell::RefCell;
use std::error::Error;
use std::sync::Mutex;
use ui;
use unicode_normalization::UnicodeNormalization;

lazy_static! {
    static ref TEXT_CACHE: Mutex<TextCache<'static>> = Mutex::new(TextCache {
        font: None,
        cache: None,
        cached_glyphs: Vec::new(),
    });
    static ref DPI_SCALE: Mutex<f32> = Mutex::new(1.0);
}

pub(crate) const GLYPH_CACHE_WIDTH: u32 = 1024;
pub(crate) const GLYPH_CACHE_HEIGHT: u32 = 1024;

type Depth = f32;

struct TextCache<'a> {
    font: Option<Font<'a>>,
    cache: Option<RefCell<Cache<'a>>>,
    cached_glyphs: Vec<(PositionedGlyph<'a>, Depth)>,
}

#[derive(Clone, Copy)]
pub enum Justify {
    Left,
    Right,
    Center,
}

pub fn initialize_font(font_data: Vec<u8>) -> Result<(), Box<Error>> {
    let cache = TextCache {
        font: Some(Font::from_bytes(font_data)?),
        cache: Some(RefCell::new(
            Cache::builder()
                .dimensions(GLYPH_CACHE_WIDTH, GLYPH_CACHE_HEIGHT)
                .build(),
        )),
        cached_glyphs: Vec::with_capacity(1000),
    };
    let mut lock = TEXT_CACHE.lock()?;
    *lock = cache;
    Ok(())
}

pub fn update_dpi(dpi: f32) {
    let mut lock = DPI_SCALE.lock().unwrap();
    *lock = dpi;
}

// TODO: Handling newlines and centering things properly on the y-axis
pub(crate) fn queue_text(
    area: ui::Rect,
    z: f32,
    font_size: f32,
    text: &str,
    justification: Justify,
) {
    let mut cache = TEXT_CACHE.lock().unwrap();
    let initial_glyphs = collect_glyphs(&mut cache, area.x0, area.y0, z, font_size, text);

    let final_glyphs;
    match justification {
        Justify::Left => final_glyphs = initial_glyphs,
        Justify::Right => {
            if let Some((width, _)) = measure_text(&initial_glyphs) {
                let xo = area.width() - width;
                final_glyphs =
                    collect_glyphs(&mut cache, area.x0 + xo, area.y0, z, font_size, text);
            } else {
                final_glyphs = initial_glyphs;
            }
        }
        Justify::Center => {
            if let Some((width, _)) = measure_text(&initial_glyphs) {
                let xo = (area.width() - width) / 2.0;
                final_glyphs =
                    collect_glyphs(&mut cache, area.x0 + xo, area.y0, z, font_size, text);
            } else {
                final_glyphs = initial_glyphs;
            }
        }
    }
    cache.cached_glyphs.extend_from_slice(&final_glyphs);
}

fn measure_text(glyphs: &Vec<(PositionedGlyph, f32)>) -> Option<(f32, f32)> {
    let mut result: Option<ui::Rect> = None;
    for (glyph, _) in glyphs {
        if let Some(glyph_rect) = glyph.pixel_bounding_box() {
            if let Some(ref mut rect) = result {
                rect.x0 = rect.x0.min(glyph_rect.min.x as f32);
                rect.y0 = rect.y0.min(glyph_rect.min.y as f32);
                rect.x1 = rect.x1.max(glyph_rect.max.x as f32);
                rect.y1 = rect.y1.max(glyph_rect.max.y as f32);
            } else {
                result = Some(ui::Rect {
                    x0: glyph_rect.min.x as f32,
                    y0: glyph_rect.min.y as f32,
                    x1: glyph_rect.max.x as f32,
                    y1: glyph_rect.max.y as f32,
                });
            }
        }
    }

    if let Some(rect) = result {
        Some((rect.x1 - rect.x0, rect.y1 - rect.y0))
    } else {
        None
    }
}

fn collect_glyphs<'a>(
    cache: &mut TextCache<'a>,
    x: f32,
    y: f32,
    z: f32,
    font_size: f32,
    text: &str,
) -> Vec<(PositionedGlyph<'a>, f32)> {
    let dpi;
    let scale;
    {
        let lock = DPI_SCALE.lock().unwrap();
        dpi = *lock;
        scale = Scale::uniform(font_size * dpi)
    }
    let x = x * dpi;
    let y = y * dpi;
    let mut glyphs = Vec::with_capacity(text.len());
    if let Some(ref font) = &cache.font {
        let v_metrics = font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = point(x, y + v_metrics.ascent);
        let mut last_glyph_id = None;
        for c in text.nfc() {
            if c.is_control() {
                if c == '\r' {
                    caret = point(x, caret.y + advance_height);
                }
                continue;
            }

            let glyph = font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += font.pair_kerning(scale, id, glyph.id());
            }
            last_glyph_id = Some(glyph.id());
            let glyph = glyph.scaled(scale).positioned(caret);
            caret.x += glyph.unpositioned().h_metrics().advance_width;
            glyphs.push((glyph, z));
        }
    }
    glyphs
}

pub(crate) fn draw_text() {
    let mut text_cache = TEXT_CACHE.lock().unwrap();

    if let Some(ref cache) = &text_cache.cache {
        let mut cache = cache.borrow_mut();

        for glyph in &text_cache.cached_glyphs {
            cache.queue_glyph(0, glyph.0.clone());
        }

        unsafe {
            let tex = TEXTURES[1];
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

        for g in &text_cache.cached_glyphs {
            if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, &g.0) {
                draw_quad(
                    screen_rect.min.x as f32 / dpi,
                    screen_rect.min.y as f32 / dpi,
                    screen_rect.max.x as f32 / dpi,
                    screen_rect.max.y as f32 / dpi,
                    g.1,
                    uv_rect.min.x,
                    uv_rect.min.y,
                    uv_rect.max.x,
                    uv_rect.max.y,
                    1,
                );
            }
        }
    }

    text_cache.cached_glyphs.clear();
}
