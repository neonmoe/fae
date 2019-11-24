use crate::error::GlyphNotRenderedError;
use crate::renderer::Renderer;
use crate::text::types::*;
use crate::text::GlyphCache;
use crate::types::*;

use rusttype::{Font, FontCollection, PositionedGlyph, Scale};
use std::collections::HashMap;

type FontSize = i32;

/// An implementation of FontProvider that uses a TTF as the font, and
/// uses `rusttype` for parsing and rasterizing it.
///
/// Contains three HashMaps for caching:
/// - Metrics, 24 bytes per GlyphId + font size combination
/// - Advance widths, 16 bytes per GlyphId pair\* + font size combination
/// - Glyph ids, 4 bytes per character
///
/// And of course, the whatever overhead each of these HashMaps will
/// have because of them being HashMaps. At least in the amount of
/// text rendered in fae's examples, these caches don't seem to have
/// much of an impact in the total memory usage of the application,
/// but they do improve performance a lot.
///
/// \* The pairs consist of each rendered glyph and the one before it.
pub struct RustTypeProvider<'a> {
    // These public variables are probably best to leave at defaults,
    // but I've left them as variables for future consideration.
    sink_overflows_into_spaces: bool,
    glyph_padding: f32,

    font: Font<'a>,
    units_per_em: i32,
    ascent: i32,
    descent: i32,
    // TODO(optimization): Unused cached values should be dropped (rusttype metric cache)
    metrics: HashMap<(GlyphId, FontSize), RectPx>,
    // TODO(optimization): Unused cached values should be dropped (rusttype advance cache)
    advances: HashMap<(GlyphId, GlyphId, FontSize), f32>,
}

impl<'a> RustTypeProvider<'a> {
    pub fn from_ttf(ttf_data: Vec<u8>) -> Result<RustTypeProvider<'a>, rusttype::Error> {
        let font = FontCollection::from_bytes(ttf_data)?.into_font()?;
        if log::log_enabled!(log::Level::Info) {
            log::info!("Loading font: {}", get_font_name(&font));
        }
        let units_per_em = font.units_per_em();
        let v_metrics = font.v_metrics_unscaled();
        Ok(RustTypeProvider {
            font,
            sink_overflows_into_spaces: false,
            glyph_padding: 0.0,
            units_per_em: i32::from(units_per_em),
            ascent: v_metrics.ascent as i32,
            descent: v_metrics.descent as i32,
            metrics: HashMap::new(),
            advances: HashMap::new(),
        })
    }

    fn font_size_to_scale(&self, font_size: i32) -> Scale {
        Scale::uniform(
            font_size as f32 * (self.ascent - self.descent) as f32 / self.units_per_em as f32,
        )
    }

    fn get_metric_from_glyph(&self, glyph: &PositionedGlyph) -> RectPx {
        if let Some(rect) = glyph.pixel_bounding_box() {
            let ascent = self.font.v_metrics(glyph.scale()).ascent;
            RectPx {
                x: rect.min.x,
                y: rect.min.y + ascent as i32,
                width: rect.width(),
                height: rect.height(),
            }
        } else {
            (0, 0, 0, 0).into()
        }
    }

    fn get_advance_from_font(&mut self, from: GlyphId, to: GlyphId, font_size: i32) -> f32 {
        let key = (from, to, font_size);
        if let Some(advance) = self.advances.get(&key) {
            *advance
        } else {
            let from = rusttype::GlyphId(from);
            let to = rusttype::GlyphId(to);
            let scale = self.font_size_to_scale(font_size);
            let from_glyph = self.font.glyph(from).scaled(scale);
            let advance =
                from_glyph.h_metrics().advance_width + self.font.pair_kerning(scale, from, to);
            self.advances.insert(key, advance);
            advance
        }
    }
}

impl<'a> FontProvider for RustTypeProvider<'a> {
    fn get_glyph_id(&mut self, c: char) -> GlyphId {
        self.font.glyph(c).id().0
    }

    fn get_line_advance(&self, cursor: Cursor, font_size: i32) -> Advance {
        let metrics = self.font.v_metrics(self.font_size_to_scale(font_size));
        let advance_y = metrics.ascent - metrics.descent + metrics.line_gap;
        Advance {
            advance_y: advance_y.trunc() as i32,
            ..Advance::from(cursor)
        }
    }

    fn get_advance(
        &mut self,
        from: GlyphId,
        to: GlyphId,
        cursor: Cursor,
        font_size: i32,
    ) -> Advance {
        let mut advance = self.get_advance_from_font(from, to, font_size) + self.glyph_padding;

        let space_accumulator = if to == self.get_glyph_id(' ') {
            advance += cursor.space_accumulator;
            0.0
        } else {
            cursor.space_accumulator
        };
        let overflow = if self.sink_overflows_into_spaces {
            advance.fract() - self.glyph_padding
        } else {
            0.0
        };

        Advance {
            advance_x: advance.trunc() as i32,
            space_accumulator: space_accumulator + overflow,
            ..Advance::from(cursor)
        }
    }

    fn get_metric(&mut self, id: GlyphId, _cursor: Cursor, font_size: i32) -> RectPx {
        let key = (id, font_size);
        if let Some(metric) = self.metrics.get(&key) {
            *metric
        } else {
            let scale = self.font_size_to_scale(font_size);
            let glyph = self
                .font
                .glyph(rusttype::GlyphId(id))
                .scaled(scale)
                .positioned(rusttype::point(0.0, 0.0));
            let metric = self.get_metric_from_glyph(&glyph);
            self.metrics.insert(key, metric);
            metric
        }
    }

    fn render_glyph(
        &mut self,
        renderer: &mut Renderer,
        cache: &mut GlyphCache,
        glyph_id: GlyphId,
        cursor: Cursor,
        font_size: i32,
    ) -> Result<RectPx, GlyphNotRenderedError> {
        let metric = self.get_metric(glyph_id, cursor, font_size);

        let id = CacheIdentifier::new(glyph_id, Some(font_size));
        let (spot, new) = cache.reserve(id, metric.width, metric.height)?;
        if new {
            let scale = self.font_size_to_scale(font_size);
            let glyph = self
                .font
                .glyph(rusttype::GlyphId(glyph_id))
                .scaled(scale)
                .positioned(rusttype::point(0.0, 0.0));

            let mut data = vec![0; (metric.width * metric.height) as usize];
            glyph.draw(|x, y, c| {
                data[(x + y * metric.width as u32) as usize] = (255.0 * c) as u8;
            });
            cache.upload_glyph(renderer, spot, |x, y| data[(x + y * metric.width) as usize]);

            crate::profiler::write(|p| p.glyph_cache_misses += 1);
        } else {
            crate::profiler::write(|p| p.glyph_cache_hits += 1);
        }
        crate::profiler::write(|p| p.glyphs_drawn += 1);
        Ok(spot)
    }
}

// Gets a name out of the font_name_strings
fn get_font_name(font: &Font) -> String {
    use stb_truetype::{MicrosoftEid, PlatformEncodingLanguageId::*, UnicodeEid};
    let mut font_name_parts = font
        .font_name_strings()
        .filter_map(|(s, plat, id)| {
            let s = std::str::from_utf8(s).ok()?;
            if id == 4
                && match plat? {
                    // I'm not sure how to interpret platform
                    // specific encodings in all situations, but
                    // this is my best guess as to which *should*
                    // be proper readable utf-8.
                    //
                    // Source: https://docs.microsoft.com/en-us/typography/opentype/spec/name#platform-ids
                    Unicode(Some(Ok(eid)), _) if eid == UnicodeEid::Unicode_2_0_Full => true,
                    Microsoft(Some(Ok(eid)), _) if eid == MicrosoftEid::UnicodeFull => true,
                    // The mac eids are just locales, and Roman works,
                    // so maybe they're all UTF-8?
                    Mac(_, _) => true,
                    _ => false,
                }
            {
                Some((s, id))
            } else {
                None
            }
        })
        .collect::<Vec<(&str, u16)>>();
    font_name_parts.dedup_by(|(_, a), (_, b)| a == b);
    debug_assert!(font_name_parts.len() == 1);
    font_name_parts[0].0.to_string()
}
