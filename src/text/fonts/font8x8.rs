use crate::text::glyph_cache::GlyphCache;
use crate::text::*;

pub struct Font8x8Provider {
    cache: GlyphCache,
}

impl Font8x8Provider {
    pub fn new(cache: GlyphCache) -> Font8x8Provider {
        Font8x8Provider { cache }
    }
}

fn get_size(font_size: f32) -> i32 {
    (font_size / 16.0).max(1.0).round() as i32 * 8
}

impl FontProvider for Font8x8Provider {
    fn get_glyph_id(&self, c: char) -> u32 {
        c as u32
    }

    fn get_line_height(&self, font_size: f32) -> i32 {
        font_size as i32
    }

    fn get_advance(&self, _from: u32, _to: u32, font_size: f32) -> Option<i32> {
        Some(get_size(font_size))
    }

    fn get_metric(&self, _id: u32, font_size: f32) -> RectPx {
        let glyph_size = get_size(font_size);
        let glyph_y = (self.get_line_height(font_size) - glyph_size) / 2;
        RectPx {
            x: 0,
            y: glyph_y,
            width: glyph_size,
            height: glyph_size,
        }
    }

    fn render_glyph(&mut self, id: u32, _font_size: f32) -> Option<RectPx> {
        use std::convert::TryFrom;
        let c = char::try_from(id).ok()?;
        if c.is_whitespace() {
            return None;
        } else {
            let bitmap = get_bitmap(c)?;
            render_bitmap(c, bitmap, &mut self.cache)
        }
    }

    fn update_glyph_cache_expiration(&mut self) {
        self.cache.expire_one_step();
    }
}

fn render_bitmap(c: char, bitmap: [u8; 8], cache: &mut GlyphCache) -> Option<RectPx> {
    let tex = cache.get_texture();
    if let Some(spot) = cache.reserve_uvs(c, 8, 8) {
        if spot.just_reserved {
            let mut data = Vec::with_capacity(8 * 8);
            for y in &bitmap {
                for x in 0..8 {
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

// This function provides glyphs for 558 characters (for calculating
// the cache texture size)
fn get_bitmap(c: char) -> Option<[u8; 8]> {
    let u = c as usize;
    match u {
        0..=0x7F => Some(font8x8::legacy::BASIC_LEGACY[u]),
        0x80..=0x9F => Some(font8x8::legacy::CONTROL_LEGACY[u - 0x80]),
        0xA0..=0xFF => Some(font8x8::legacy::LATIN_LEGACY[u - 0xA0]),
        0x2500..=0x257F => Some(font8x8::legacy::BOX_LEGACY[u - 0x2500]),
        0x2580..=0x259F => Some(font8x8::legacy::BLOCK_LEGACY[u - 0x2580]),
        0x3040..=0x309F => Some(font8x8::legacy::HIRAGANA_LEGACY[u - 0x3040]),
        // TODO: The 'micro' glyph doesn't seem to work, debug
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
    }
}

#[test]
fn get_font8x8_bitmap_works() {
    for u in 0..0xFFFF as u32 {
        use std::convert::TryFrom;
        if let Ok(c) = char::try_from(u) {
            get_bitmap(c);
        }
    }
}
