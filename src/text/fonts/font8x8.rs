use crate::error::GlyphNotRenderedError;
use crate::renderer::Renderer;
use crate::text::glyph_cache::GlyphCache;
use crate::text::types::*;
use crate::text::*;

use fnv::FnvHashMap;

/// An implementation of FontProvider that uses `font8x8` as the font.
///
/// Contains a metric cache in the form of a HashMap, which takes up
/// 17 bytes per glyph (1 byte from the GlyphId, 16 bytes from the
/// RectPx) plus the HashMap's overhead. This cache improves
/// performance a lot, as the metrics are accessed multiple times per
/// glyph, every frame in the worst case. As no glyphs in the font8x8
/// font rely on font size, this cache can't get very big.
pub struct Font8x8Provider {
    metrics: FnvHashMap<GlyphId, RectPx>,
}

impl Font8x8Provider {
    pub fn new() -> Font8x8Provider {
        Font8x8Provider {
            metrics: FnvHashMap::default(),
        }
    }
}

impl FontProvider for Font8x8Provider {
    fn get_glyph_id(&mut self, c: char) -> GlyphId {
        c as GlyphId
    }

    fn get_line_advance(&self, font_size: i32) -> Advance {
        Advance {
            advance_x: 0,
            advance_y: font_size + 3,
        }
    }

    fn get_advance(&mut self, from: GlyphId, _to: GlyphId, font_size: i32) -> Advance {
        let RectPx { width, .. } = self.get_raw_metrics(from);
        Advance {
            advance_x: scale(width, font_size) + 1,
            advance_y: 0,
        }
    }

    fn get_metric(&mut self, id: GlyphId, font_size: i32) -> RectPx {
        let scaled_line_height = self.get_line_advance(font_size).advance_y;
        let y_offset = (scaled_line_height - scale(8, font_size)) / 2;

        let metrics = self.get_raw_metrics(id);
        let metrics_y = scale_f32(metrics.y as f32, font_size as f32);
        let metrics_height = scale_f32(metrics.height as f32, font_size as f32);
        let metrics_height = (metrics_height + metrics_y.fract()).trunc() as i32;
        let metrics_y = metrics_y.trunc() as i32;

        RectPx {
            x: 0,
            y: y_offset + metrics_y,
            width: scale(metrics.width, font_size),
            height: metrics_height,
        }
    }

    fn render_glyph(
        &mut self,
        renderer: &mut Renderer,
        cache: &mut GlyphCache,
        id: GlyphId,
        _font_size: i32,
    ) -> Result<RectPx, GlyphNotRenderedError> {
        if let Some(bitmap) = get_bitmap(id) {
            self.render_bitmap(renderer, cache, id, bitmap)
        } else {
            self.render_bitmap(renderer, cache, id, get_missing_bitmap())
        }
    }
}

impl Font8x8Provider {
    fn get_raw_metrics(&mut self, id: GlyphId) -> RectPx {
        if id == ' ' as GlyphId {
            (0, 8, 3, 0).into()
        } else if let Some(metric) = self.metrics.get(&id) {
            *metric
        } else {
            let (left, right) = get_empty_pixels_left_right(id).unwrap_or((0, 0));
            let (top, bottom) = get_empty_pixels_top_bottom(id).unwrap_or((0, 0));
            let metric = RectPx {
                x: left,
                y: top,
                width: 8 - (left + right),
                height: 8 - (top + bottom),
            };
            self.metrics.insert(id, metric);
            metric
        }
    }

    fn render_bitmap(
        &mut self,
        renderer: &mut Renderer,
        cache: &mut GlyphCache,
        id: GlyphId,
        bitmap: [u8; 8],
    ) -> Result<RectPx, GlyphNotRenderedError> {
        let metric = self.get_raw_metrics(id);

        let id = CacheIdentifier::new(id, None);
        let (spot, new) = cache.reserve(id, metric.width, metric.height)?;
        if new {
            let x_offset = metric.x;
            let y_offset = metric.y;
            cache.upload_glyph(renderer, spot, |x, y| {
                if (bitmap[(y + y_offset) as usize] & (1 << (x + x_offset))) == 0 {
                    0x0
                } else {
                    0xFF
                }
            });

            crate::profiler::write(|p| p.glyph_cache_misses += 1);
        } else {
            crate::profiler::write(|p| p.glyph_cache_hits += 1);
        }
        crate::profiler::write(|p| p.glyphs_drawn += 1);
        Ok(spot)
    }
}

fn scale(i: i32, font_size: i32) -> i32 {
    i * font_size / 10
}

fn scale_f32(i: f32, font_size: f32) -> f32 {
    i * font_size / 10.0
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
    for (i, y) in bitmap.iter().enumerate() {
        if *y != 0 {
            if top.is_none() {
                top = Some(i);
            }
            bottom = Some(i);
        }
    }
    Some((top? as i32, 7 - bottom? as i32))
}

fn get_missing_bitmap() -> [u8; 8] {
    // Produces a bitmap of a rectangle, looks like:
    // ........
    // .######.
    // .#....#.
    // .#....#.
    // .#....#.
    // .######.
    // ........
    // ........
    [0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x0]
}

// This function provides glyphs for 558 characters (for calculating
// the cache texture size)
#[doc(hidden)]
pub fn get_bitmap(id: GlyphId) -> Option<[u8; 8]> {
    let u = id as usize;
    let bitmap = match u {
        0 => Some(get_missing_bitmap()), // Standard missing glyph id
        1..=0x7F => Some(font8x8::legacy::BASIC_LEGACY[u]),
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
