// TODO: Move text processing from draw_text to compose_draw_call
use crate::gl;
use crate::gl::types::*;
use crate::image::Image;
use crate::renderer::{DrawCallHandle, DrawCallParameters, Renderer, Shaders};
use rusttype::gpu_cache::Cache;
use rusttype::*;
use std::cell::RefCell;
use std::error::Error;
use unicode_normalization::UnicodeNormalization;

pub(crate) const GLYPH_CACHE_WIDTH: u32 = 1024;
pub(crate) const GLYPH_CACHE_HEIGHT: u32 = 1024;

const DEFAULT_TEXT_SHADERS: Shaders = Shaders {
    vertex_shader_110: include_str!("shaders/legacy/texquad.vert"),
    fragment_shader_110: include_str!("shaders/legacy/text.frag"),
    vertex_shader_330: include_str!("shaders/texquad.vert"),
    fragment_shader_330: include_str!("shaders/text.frag"),
};

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

struct TextRender {
    glyphs: Vec<SizedGlyph>,
    clip_area: Option<(f32, f32, f32, f32)>,
    z: f32,
}

#[derive(Clone)]
struct SizedGlyph {
    glyph: PositionedGlyph<'static>,
    width: f32,
}

pub struct TextRenderer {
    font: Font<'static>,
    cache: RefCell<Cache<'static>>,
    cached_text: Vec<TextRender>,
    dpi_factor: f32,
    draw_call: DrawCallHandle,
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
        font_data: Vec<u8>,
        subpixel_accurate: bool,
        renderer: &mut Renderer,
    ) -> Result<TextRenderer, Box<Error>> {
        let glyph_cache_image =
            Image::from_color(GLYPH_CACHE_WIDTH as i32, GLYPH_CACHE_HEIGHT as i32, &[0])
                .format(gl::RED);
        let params = DrawCallParameters {
            image: Some(glyph_cache_image),
            shaders: Some(DEFAULT_TEXT_SHADERS),
            ..Default::default()
        };
        let draw_call = renderer.create_draw_call(params);
        let position_tolerance = if subpixel_accurate { 0.1 } else { 1.0 };

        Ok(TextRenderer {
            font: Font::from_bytes(font_data)?,
            cache: RefCell::new(
                Cache::builder()
                    .dimensions(GLYPH_CACHE_WIDTH, GLYPH_CACHE_HEIGHT)
                    .position_tolerance(position_tolerance)
                    .build(),
            ),
            cached_text: Vec::new(),
            dpi_factor: 1.0,
            draw_call,
        })
    }

    pub fn update_dpi_factor(&mut self, dpi_factor: f32) {
        self.dpi_factor = dpi_factor;
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        (x, y, z): (f32, f32, f32),
        font_size: f32,
        alignment: Alignment,
        max_row_width: Option<f32>,
        clip_area: Option<(f32, f32, f32, f32)>,
    ) {
        let rows = self.collect_glyphs(x, y, max_row_width, font_size, text);
        let dpi = self.dpi_factor;

        let mut final_glyphs = Vec::with_capacity(text.len());

        // Collect the rows and offset them according to the alignment
        if let Some(width) = max_row_width {
            match alignment {
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

                Alignment::Left => {
                    for row in rows {
                        final_glyphs.extend_from_slice(&row);
                    }
                }
            }
        } else {
            for row in rows {
                final_glyphs.extend_from_slice(&row);
            }
        }

        self.cached_text.push(TextRender {
            glyphs: final_glyphs,
            clip_area,
            z,
        });
    }

    fn collect_glyphs(
        &self,
        x: f32,
        y: f32,
        width: Option<f32>,
        font_size: f32,
        text: &str,
    ) -> Vec<Vec<SizedGlyph>> {
        let dpi = self.dpi_factor;
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

            let glyph = self.font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += self.font.pair_kerning(scale, id, glyph.id());
            }

            if width.is_some() && caret.x > (x + width.unwrap()) * dpi {
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

    pub fn compose_draw_call(&mut self, renderer: &mut Renderer) {
        let &mut TextRenderer {
            dpi_factor,
            ref draw_call,
            ..
        } = self;
        let mut cache = self.cache.borrow_mut();

        for text in &self.cached_text {
            for glyph in &text.glyphs {
                cache.queue_glyph(0, glyph.glyph.clone());
            }
        }

        let tex = renderer.get_texture(draw_call);
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

            let clip_coords;
            let clipped;
            if let Some(clip_area) = text.clip_area {
                clip_coords = clip_area;
                clipped = true;
            } else {
                clip_coords = (0.0, 0.0, 0.0, 0.0);
                clipped = false;
            }
            for glyph in &text.glyphs {
                if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, &glyph.glyph) {
                    let coords = (
                        screen_rect.min.x as f32 / dpi_factor,
                        screen_rect.min.y as f32 / dpi_factor,
                        screen_rect.max.x as f32 / dpi_factor,
                        screen_rect.max.y as f32 / dpi_factor,
                    );
                    let texcoords = (uv_rect.min.x, uv_rect.min.y, uv_rect.max.x, uv_rect.max.y);
                    if clipped {
                        renderer.draw_quad_clipped(
                            clip_coords,
                            coords,
                            texcoords,
                            (0.0, 0.0, 0.0, 1.0),
                            (0.0, 0.0, 0.0),
                            z,
                            draw_call,
                        );
                    } else {
                        renderer.draw_quad(
                            coords,
                            texcoords,
                            (0.0, 0.0, 0.0, 1.0),
                            (0.0, 0.0, 0.0),
                            z,
                            draw_call,
                        );
                    };
                }
            }
        }

        self.cached_text.clear();
    }
}

/// Will only return `None` when `index >= glyphs.len()`.
fn measure_text_at_index(
    glyphs: &[SizedGlyph],
    index: usize,
    dpi: f32,
) -> Option<(f32, f32, f32, f32)> {
    if index >= glyphs.len() {
        return None;
    }

    let width = glyphs[index].width;
    let glyph = &glyphs[index].glyph;
    let position = glyph.position();
    if let Some(rect) = glyph.pixel_bounding_box() {
        return Some((
            rect.min.x as f32 / dpi,
            rect.min.y as f32 / dpi,
            rect.max.x as f32 / dpi,
            rect.max.y as f32 / dpi,
        ));
    } else {
        let (x, y) = (position.x / dpi, position.y / dpi);
        return Some((x, y, x + width / dpi, y + 1.0));
    }
}

fn measure_text(glyphs: &[SizedGlyph], dpi: f32) -> Option<(f32, f32)> {
    let mut result: Option<(f32, f32, f32, f32)> = None;

    for i in 0..glyphs.len() {
        if let Some(glyph_rect) = measure_text_at_index(glyphs, i, dpi) {
            if let Some(ref mut rect) = result {
                *rect = (
                    rect.0.min(glyph_rect.0),
                    rect.1.min(glyph_rect.1),
                    rect.2.max(glyph_rect.2),
                    rect.3.max(glyph_rect.3),
                );
            } else {
                result = Some(glyph_rect);
            }
        }
    }

    if let Some(rect) = result {
        Some((rect.2 - rect.0, rect.3 - rect.1))
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
