use crate::error::GlyphNotRenderedError;
use crate::text::glyph_cache::GlyphCache;
use crate::text::types::*;
use crate::text::*;

use std::cell::RefCell;
use std::collections::HashMap;

fn scale(i: i32, font_size: i32) -> i32 {
    i * font_size / 8
}

pub struct Font8x8Provider {
    metrics: RefCell<HashMap<GlyphId, RectPx>>,
}

impl Font8x8Provider {
    pub fn new() -> Font8x8Provider {
        Font8x8Provider {
            metrics: RefCell::new(HashMap::new()),
        }
    }
}

impl FontProvider for Font8x8Provider {
    fn get_glyph_id(&self, c: char) -> Option<GlyphId> {
        Some(c as GlyphId)
    }

    fn get_line_height(&self, font_size: i32) -> i32 {
        font_size * 4 / 3
    }

    fn get_advance(&self, from: GlyphId, _to: GlyphId, font_size: i32) -> Option<i32> {
        let RectPx { width, .. } = self.get_raw_metrics(from);
        Some((width * font_size / 8 + 1) as i32)
    }

    fn get_metric(&self, id: GlyphId, font_size: i32) -> RectPx {
        let metrics = self.get_raw_metrics(id);
        let y_offset = (self.get_line_height(font_size) / font_size * 8 - 8) / 2;
        RectPx {
            x: 0,
            y: scale(y_offset + metrics.y, font_size),
            width: scale(metrics.width, font_size),
            height: scale(metrics.height, font_size),
        }
    }

    fn render_glyph(
        &mut self,
        cache: &mut GlyphCache,
        id: GlyphId,
        _font_size: i32,
    ) -> Result<RectPx, GlyphNotRenderedError> {
        if let Some(bitmap) = get_bitmap(id) {
            self.render_bitmap(cache, id, bitmap)
        } else {
            Err(GlyphNotRenderedError::GlyphInvisible)
        }
    }
}

impl Font8x8Provider {
    fn get_raw_metrics(&self, id: GlyphId) -> RectPx {
        if id == ' ' as GlyphId {
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

    fn render_bitmap(
        &mut self,
        cache: &mut GlyphCache,
        id: GlyphId,
        bitmap: [u8; 8],
    ) -> Result<RectPx, GlyphNotRenderedError> {
        let metric = self.get_raw_metrics(id);

        let id = CacheIdentifier::new(id, 0);
        let tex = cache.get_texture();
        let (spot, new) = cache.reserve_uvs(id, metric.width, metric.height)?;
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
        Ok(spot)
    }
}

fn get_empty_pixels_left_right(id: GlyphId) -> Option<(i32, i32)> {
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

fn get_empty_pixels_top_bottom(id: GlyphId) -> Option<(i32, i32)> {
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
pub fn get_bitmap(id: GlyphId) -> Option<[u8; 8]> {
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
    for u in 0..0xFFFF as GlyphId {
        get_bitmap(u);
    }
}
