use crate::text::glyph_cache::GlyphCache;
use crate::text::types::*;
use crate::text::*;

use std::cell::RefCell;
use std::collections::HashMap;

fn scale(i: i32, font_size: f32) -> i32 {
    (i as f32 * font_size / 8.0) as i32
}

pub struct Font8x8Provider {
    cache: GlyphCache,
    metrics: RefCell<HashMap<u32, RectPx>>,
}

impl Font8x8Provider {
    pub fn new(cache: GlyphCache) -> Font8x8Provider {
        Font8x8Provider {
            cache,
            metrics: RefCell::new(HashMap::new()),
        }
    }
}

impl Font8x8Provider {
    fn get_raw_metrics(&self, id: u32) -> RectPx {
        if id == ' ' as u32 {
            (0, 8, 3, 0).into()
        } else {
            *self.metrics.borrow_mut().entry(id).or_insert_with(|| {
                let (left, right) = get_empty_pixels_left_right(id).unwrap_or((0, 0));
                let (top, bottom) = get_empty_pixels_top_bottom(id).unwrap_or((0, 0));
                RectPx {
                    x: left,
                    y: top,
                    width: 8 - (left + right),
                    height: 8 - (top + bottom),
                }
            })
        }
    }

    fn render_bitmap(&mut self, id: u32, bitmap: [u8; 8]) -> Option<RectPx> {
        let metric = self.get_raw_metrics(id);

        use std::convert::TryFrom;
        let id = CacheIdentifier::new(char::try_from(id).ok()?);
        let tex = self.cache.get_texture();
        if let Some((spot, new)) = self.cache.reserve_uvs(id, metric.width, metric.height) {
            if new {
                let mut data = Vec::with_capacity((metric.width * metric.height) as usize);
                for y in metric.y..(metric.y + metric.height) {
                    let color = bitmap[y as usize];
                    for x in metric.x..(metric.x + metric.width) {
                        if (color & (1 << x)) == 0 {
                            data.push(0x00u8);
                        } else {
                            data.push(0xFFu8);
                        }
                    }
                }

                unsafe {
                    use crate::gl;
                    use crate::gl::types::*;
                    gl::BindTexture(gl::TEXTURE_2D, tex);
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                    gl::TexSubImage2D(
                        gl::TEXTURE_2D,            // target
                        0,                         // level
                        spot.x,                    // xoffset
                        spot.y,                    // yoffset
                        spot.width,                // width
                        spot.height,               // height
                        gl::RED as GLuint,         // format
                        gl::UNSIGNED_BYTE,         // type
                        data.as_ptr() as *const _, // pixels
                    );
                    crate::renderer::print_gl_errors("after font8x8 render_bitmap texsubimage2d");
                }
                crate::profiler::write(|p| p.glyph_cache_misses += 1);
            } else {
                crate::profiler::write(|p| p.glyph_cache_hits += 1);
            }
            crate::profiler::write(|p| p.glyphs_drawn += 1);
            Some(spot)
        } else {
            None
        }
    }
}

impl FontProvider for Font8x8Provider {
    fn get_glyph_id(&self, c: char) -> u32 {
        c as u32
    }

    fn get_line_height(&self, font_size: f32) -> f32 {
        font_size * 4.0 / 3.0
    }

    fn get_advance(&self, from: u32, _to: u32, font_size: f32) -> Option<i32> {
        let RectPx { width, .. } = self.get_raw_metrics(from);
        Some((width as f32 * font_size / 8.0 + 1.0) as i32)
    }

    fn get_metric(&self, id: u32, font_size: f32) -> RectPx {
        let metrics = self.get_raw_metrics(id);
        let y_offset = (self.get_line_height(font_size) as i32 - scale(8, font_size)) / 2;
        RectPx {
            x: 0,
            // TODO: Something is still wrong with this y
            y: y_offset + scale(metrics.y, font_size),
            width: scale(metrics.width, font_size),
            height: scale(metrics.height, font_size),
        }
    }

    fn render_glyph(&mut self, id: u32, _font_size: f32) -> Option<RectPx> {
        if id == ' ' as u32 {
            None
        } else {
            let bitmap = get_bitmap(id)?;
            self.render_bitmap(id, bitmap)
        }
    }

    fn update_glyph_cache_expiration(&mut self) {
        self.cache.expire_one_step();
    }
}

fn get_empty_pixels_left_right(id: u32) -> Option<(i32, i32)> {
    let bitmap = get_bitmap(id)?;
    let mut left = None;
    let mut right = None;
    for y in &bitmap {
        for x in 0..8 {
            if (y & (1 << x)) != 0 {
                if x < left.unwrap_or(8) {
                    left = Some(x);
                }
                if x > right.unwrap_or(-1) {
                    right = Some(x);
                }
            }
        }
    }
    Some((left?, 7 - right?))
}

fn get_empty_pixels_top_bottom(id: u32) -> Option<(i32, i32)> {
    let bitmap = get_bitmap(id)?;
    let mut top = None;
    let mut bottom = None;
    for i in 0..bitmap.len() {
        let (i, y) = (i as i32, bitmap[i]);

        if y != 0 {
            if top.is_none() {
                top = Some(i);
            }
            bottom = Some(i);
        }
    }
    Some((top?, 7 - bottom?))
}

// This function provides glyphs for 558 characters (for calculating
// the cache texture size)
#[doc(hidden)]
pub fn get_bitmap(id: u32) -> Option<[u8; 8]> {
    let u = id as usize;
    let bitmap = match u {
        0..=0x7F => Some(font8x8::legacy::BASIC_LEGACY[u]),
        0x80..=0x9F => Some(font8x8::legacy::CONTROL_LEGACY[u - 0x80]),
        0xA0..=0xFF => Some(font8x8::legacy::LATIN_LEGACY[u - 0xA0]),
        0x2500..=0x257F => Some(font8x8::legacy::BOX_LEGACY[u - 0x2500]),
        0x2580..=0x259F => Some(font8x8::legacy::BLOCK_LEGACY[u - 0x2580]),
        0x3040..=0x309F => Some(font8x8::legacy::HIRAGANA_LEGACY[u - 0x3040]),
        0x390..=0x03C9 => Some(font8x8::legacy::GREEK_LEGACY[u - 0x390]),
        0xE541..=0xE55A => Some(font8x8::legacy::SGA_LEGACY[u - 0xE541]),
        0x20A7 => Some(font8x8::legacy::MISC_LEGACY[0]),
        0x192 => Some(font8x8::legacy::MISC_LEGACY[1]),
        0x2310 => Some(font8x8::legacy::MISC_LEGACY[4]),
        0x2264 => Some(font8x8::legacy::MISC_LEGACY[5]),
        0x2265 => Some(font8x8::legacy::MISC_LEGACY[6]),
        0x1EF2 => Some(font8x8::legacy::MISC_LEGACY[8]),
        0x1EF3 => Some(font8x8::legacy::MISC_LEGACY[9]),
        // The following are covered by BASIC and LATIN
        //0xAA => Some(font8x8::legacy::MISC_LEGACY[2]),
        //0xBA => Some(font8x8::legacy::MISC_LEGACY[3]),
        //0x60 => Some(font8x8::legacy::MISC_LEGACY[7]),
        _ => None,
    }?;

    // Since whitespace and control characters have empty bitmaps in
    // font8x8, we need to ensure that the bitmap is not empty.
    for y in &bitmap {
        if *y != 0 {
            return Some(bitmap);
        }
    }

    None
}

#[test]
fn get_font8x8_bitmap_works() {
    for u in 0..0xFFFF as u32 {
        get_bitmap(u);
    }
}
