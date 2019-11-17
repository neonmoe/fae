use crate::error::GlyphNotRenderedError;
use crate::text::types::*;
use crate::text::GlyphCache;
use crate::types::*;

use rusttype::{Font, FontCollection, PositionedGlyph, Scale};
use std::collections::HashMap;

pub struct RustTypeProvider<'a> {
    font: Font<'a>,
    units_per_em: i32,
    ascent: i32,
    descent: i32,
    metrics: HashMap<(GlyphId, i32), RectPx>,
}

impl<'a> RustTypeProvider<'a> {
    pub fn from_ttf(ttf_data: Vec<u8>) -> Result<RustTypeProvider<'a>, rusttype::Error> {
        let font = FontCollection::from_bytes(ttf_data)?.into_font()?;
        let units_per_em = font.units_per_em();
        let v_metrics = font.v_metrics_unscaled();
        Ok(RustTypeProvider {
            font,
            units_per_em: i32::from(units_per_em),
            ascent: v_metrics.ascent as i32,
            descent: v_metrics.descent as i32,
            metrics: HashMap::new(),
        })
    }

    fn font_size_to_scale(&self, font_size: i32) -> Scale {
        Scale::uniform(
            font_size as f32 * (self.ascent - self.descent) as f32 / self.units_per_em as f32,
        )
    }
}

impl<'a> FontProvider for RustTypeProvider<'a> {
    fn get_glyph_id(&self, c: char) -> Option<GlyphId> {
        let glyph = self.font.glyph(c);
        let id = glyph.id().0;
        if id == 0 {
            None
        } else {
            Some(id)
        }
    }

    fn get_line_height(&self, font_size: i32) -> i32 {
        // Note: this does take about a microsecond, so it would be
        // faster with a cache, but this isn't called that often.
        let metrics = self.font.v_metrics(self.font_size_to_scale(font_size));
        (metrics.ascent - metrics.descent + metrics.line_gap) as i32
    }

    fn get_advance(&self, from: GlyphId, to: GlyphId, font_size: i32) -> Option<i32> {
        let from = rusttype::GlyphId(from);
        let to = rusttype::GlyphId(to);
        let scale = self.font_size_to_scale(font_size);
        let glyph = self.font.glyph(from).scaled(scale);
        let from_width = glyph.h_metrics().advance_width;
        let kern = self.font.pair_kerning(scale, from, to);
        Some((from_width + kern).round() as i32)
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
                // TODO: Add some subpixel accuracy?
                .positioned(rusttype::point(0.0, 0.0));
            let metric = get_metric_from_glyph(&glyph);
            self.metrics.insert(key, metric);
            metric
        }
    }

    fn render_glyph(
        &mut self,
        cache: &mut GlyphCache,
        id: GlyphId,
        font_size: i32,
    ) -> Result<RectPx, GlyphNotRenderedError> {
        let scale = self.font_size_to_scale(font_size);
        let glyph = self
            .font
            .glyph(rusttype::GlyphId(id))
            .scaled(scale)
            // TODO: Add some subpixel accuracy?
            .positioned(rusttype::point(0.0, 0.0));
        let metric = get_metric_from_glyph(&glyph);

        let id = CacheIdentifier::new(id, 0);
        let tex = cache.get_texture();
        let (spot, new) = cache.reserve_uvs(id, metric.width, metric.height)?;
        if new {
            // TODO: Add borders around glyphs in the glyph cache when rendering to avoid needing to clear the texture
            let mut data = vec![0; (metric.width * metric.height) as usize];
            let stride = metric.width as u32;
            glyph.draw(|x, y, c| {
                data[(x + y * stride) as usize] = (255.0 * c) as u8;
            });

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
                crate::renderer::print_gl_errors("after rusttype render_glyph texsubimage2d");
            }
            crate::profiler::write(|p| p.glyph_cache_misses += 1);
        } else {
            crate::profiler::write(|p| p.glyph_cache_hits += 1);
        }
        crate::profiler::write(|p| p.glyphs_drawn += 1);
        Ok(spot)
    }
}

fn get_metric_from_glyph(glyph: &PositionedGlyph) -> RectPx {
    if let Some(rect) = glyph.pixel_bounding_box() {
        RectPx {
            x: rect.min.x,
            y: rect.min.y,
            width: rect.width(),
            height: rect.height(),
        }
    } else {
        (0, 0, 0, 0).into()
    }
}
