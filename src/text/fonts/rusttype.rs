use crate::renderer::Renderer;
use crate::text::types::*;
use crate::text::GlyphCache;
use crate::types::*;

use fnv::FnvHashMap;
use rusttype::{Font, FontCollection, Scale};

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
pub(crate) struct RustTypeProvider<'a> {
    glyph_padding: f32,
    font: Font<'a>,
    units_per_em: i32,
    ascent: i32,
    descent: i32,
    metrics: FnvHashMap<(GlyphId, FontSize), RectPx>,
    advances: FnvHashMap<(GlyphId, GlyphId, FontSize), f32>,
}

impl<'a> RustTypeProvider<'a> {
    pub fn new(ttf_data: Vec<u8>) -> Result<RustTypeProvider<'a>, rusttype::Error> {
        let font = FontCollection::from_bytes(ttf_data)?.into_font()?;
        if log::log_enabled!(log::Level::Info) {
            log::info!("Loading font: {}", get_font_name(&font));
        }
        let units_per_em = font.units_per_em();
        let v_metrics = font.v_metrics_unscaled();
        Ok(RustTypeProvider {
            font,
            glyph_padding: 0.0,
            units_per_em: i32::from(units_per_em),
            ascent: v_metrics.ascent as i32,
            descent: v_metrics.descent as i32,
            metrics: FnvHashMap::default(),
            advances: FnvHashMap::default(),
        })
    }

    fn font_size_to_scale(&self, font_size: i32) -> Scale {
        Scale::uniform(
            font_size as f32 * (self.ascent - self.descent) as f32 / self.units_per_em as f32,
        )
    }
}

impl<'a> FontProvider for RustTypeProvider<'a> {
    fn get_glyph_id(&mut self, c: char) -> GlyphId {
        self.font.glyph(c).id().0
    }

    fn get_line_advance(&self, font_size: i32) -> Advance {
        let metrics = self.font.v_metrics(self.font_size_to_scale(font_size));
        let advance_y = metrics.ascent - metrics.descent + metrics.line_gap;
        Advance {
            advance_x: 0,
            advance_y: advance_y.trunc() as i32,
        }
    }

    fn get_advance(&mut self, from: GlyphId, to: GlyphId, font_size: i32) -> Advance {
        let key = (from, to, font_size);
        let advance = if let Some(advance) = self.advances.get(&key) {
            *advance
        } else {
            let from = rusttype::GlyphId(from);
            let to = rusttype::GlyphId(to);
            let scale = self.font_size_to_scale(font_size);
            let from_glyph = self.font.glyph(from).scaled(scale);
            let kern = self.font.pair_kerning(scale, from, to);
            let advance = from_glyph.h_metrics().advance_width + kern;
            self.advances.insert(key, advance);
            advance
        };

        Advance {
            advance_x: (advance + self.glyph_padding).trunc() as i32,
            advance_y: 0,
        }
    }

    fn get_metric(&mut self, id: GlyphId, font_size: i32) -> RectPx {
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
            let metric = if let Some(rect) = glyph.pixel_bounding_box() {
                let ascent = self.font.v_metrics(glyph.scale()).ascent;
                RectPx {
                    x: rect.min.x,
                    y: rect.min.y + ascent as i32,
                    width: rect.width(),
                    height: rect.height(),
                }
            } else {
                (0, 0, 0, 0).into()
            };
            self.metrics.insert(key, metric);
            metric
        }
    }

    fn render_glyph(
        &mut self,
        renderer: &mut Renderer,
        cache: &mut GlyphCache,
        glyph_id: GlyphId,
        font_size: i32,
    ) -> Result<RectPx, GlyphRenderingError> {
        let metric = self.get_metric(glyph_id, font_size);

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
