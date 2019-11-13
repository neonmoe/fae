use crate::text::glyph_cache::{CacheIdentifier, GlyphCache};
use crate::text::*;

pub struct Font8x8Provider {
    cache: GlyphCache,
}

impl Font8x8Provider {
    pub fn new(cache: GlyphCache) -> Font8x8Provider {
        Font8x8Provider { cache }
    }
}

#[inline]
fn get_size(font_size: f32) -> i32 {
    font_size as i32
}

#[inline]
fn get_width(id: u32, font_size: f32) -> f32 {
    let (left, right) = get_empty_pixels_sides(id).unwrap_or((0, 0));
    if id == ' ' as u32 {
        font_size * 2.0 / 3.0 - 2.0
    } else {
        font_size - (left + right) as f32 * font_size / 8.0
    }
}

impl FontProvider for Font8x8Provider {
    fn get_glyph_id(&self, c: char) -> u32 {
        c as u32
    }

    fn get_line_height(&self, font_size: f32) -> i32 {
        get_size(font_size) * 4 / 3
    }

    fn get_advance(&self, from: u32, _to: u32, font_size: f32) -> Option<i32> {
        Some((get_width(from, font_size) + 1.0).round() as i32)
    }

    fn get_metric(&self, id: u32, font_size: f32) -> RectPx {
        let size = get_size(font_size);
        let y = (self.get_line_height(font_size) - size) / 2;
        (0, y, get_width(id, font_size).round() as i32, size).into()
    }

    fn render_glyph(&mut self, id: u32, _font_size: f32) -> Option<RectPx> {
        if id == ' ' as u32 {
            None
        } else {
            let bitmap = get_bitmap(id)?;
            render_bitmap(id, bitmap, &mut self.cache)
        }
    }

    fn update_glyph_cache_expiration(&mut self) {
        self.cache.expire_one_step();
    }
}

fn render_bitmap(id: u32, bitmap: [u8; 8], cache: &mut GlyphCache) -> Option<RectPx> {
    let (left, right) = get_empty_pixels_sides(id).unwrap_or((0, 0));
    let (width, height) = ((8 - (left + right)), 8);

    use std::convert::TryFrom;
    let c = char::try_from(id).ok()?;
    let tex = cache.get_texture();
    if let Some(spot) = cache.reserve_uvs(CacheIdentifier::new(c), width, height) {
        if spot.just_reserved {
            let mut data = Vec::with_capacity((width * height) as usize);
            for y in &bitmap {
                for x in left..(8 - right) {
                    if (y & (1 << x)) == 0 {
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
                    spot.texcoords.x,          // xoffset
                    spot.texcoords.y,          // yoffset
                    spot.texcoords.width,      // width
                    spot.texcoords.height,     // height
                    gl::RED as GLuint,         // format
                    gl::UNSIGNED_BYTE,         // type
                    data.as_ptr() as *const _, // pixels
                );
                crate::renderer::print_gl_errors("after font8x8 render_bitmap texsubimage2d");
            }
            crate::profiler::modify_profiler_value_i32("glyphs drawn", |i| i + 1);
        }
        crate::profiler::modify_profiler_value_i32("glyphs rendered", |i| i + 1);
        Some(spot.texcoords)
    } else {
        None
    }
}

fn get_empty_pixels_sides(id: u32) -> Option<(i32, i32)> {
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
        // TODO: Test all of the font8x8 glyphs, draw a grid or something
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
