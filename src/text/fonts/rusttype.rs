use crate::error::GlyphNotRenderedError;
use crate::text::types::*;
use crate::text::GlyphCache;
use crate::types::*;

use rusttype::{Font, FontCollection, PositionedGlyph, Scale};
use std::collections::HashMap;

type FontSize = i32;

pub struct RustTypeProvider<'a> {
    font: Font<'a>,
    units_per_em: i32,
    ascent: i32,
    descent: i32,
    metrics: HashMap<(GlyphId, FontSize, SubpixelOffset), RectPx>,
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

    fn get_line_advance(&self, cursor: Cursor, font_size: i32) -> Advance {
        let metrics = self.font.v_metrics(self.font_size_to_scale(font_size));
        let advance = (metrics.ascent - metrics.descent
            + metrics.line_gap
            + cursor.leftover_y.trunc()) as i32;
        Advance::new(0, advance, cursor.leftover_x, cursor.leftover_y.fract())
    }

    fn get_advance(&self, from: GlyphId, to: GlyphId, cursor: Cursor, font_size: i32) -> Advance {
        let from = rusttype::GlyphId(from);
        let to = rusttype::GlyphId(to);
        let subpixel = cursor.subpixel_offset();
        let scale = self.font_size_to_scale(font_size);
        let from_glyph = self.font.glyph(from).scaled(scale);
        let from_glyph = from_glyph.positioned(subpixel.into());
        let from_advance = from_glyph
            .pixel_bounding_box()
            .map(|rect| (rect.min.x + rect.width()) as f32)
            .unwrap_or_else(|| from_glyph.unpositioned().h_metrics().advance_width);
        let kern = self.font.pair_kerning(scale, from, to);
        let advance = from_advance + kern;
        // FIXME: Kerning seems weird in places (might be fixed, needs testing)
        Advance::new(advance.trunc() as i32, 0, 0.0, cursor.leftover_y)
    }

    fn get_metric(&mut self, id: GlyphId, cursor: Cursor, font_size: i32) -> RectPx {
        let subpixel = cursor.subpixel_offset();
        let key = (id, font_size, subpixel);
        if let Some(metric) = self.metrics.get(&key) {
            *metric
        } else {
            let scale = self.font_size_to_scale(font_size);
            let glyph = self
                .font
                .glyph(rusttype::GlyphId(id))
                .scaled(scale)
                .positioned(subpixel.into());
            let metric = self.get_metric_from_glyph(&glyph);
            self.metrics.insert(key, metric);
            metric
        }
    }

    fn render_glyph(
        &mut self,
        cache: &mut GlyphCache,
        glyph_id: GlyphId,
        cursor: Cursor,
        font_size: i32,
    ) -> Result<RectPx, GlyphNotRenderedError> {
        let subpixel = cursor.subpixel_offset();
        let metric = self.get_metric(glyph_id, cursor, font_size);

        let id = CacheIdentifier::new(glyph_id, Some(font_size), Some(subpixel));
        let tex = cache.get_texture();
        let (spot, new) = cache.reserve_uvs(id, metric.width, metric.height)?;
        if new {
            // TODO: Add borders around glyphs in the glyph cache when rendering to avoid needing to clear the texture
            let scale = self.font_size_to_scale(font_size);
            let glyph = self
                .font
                .glyph(rusttype::GlyphId(glyph_id))
                .scaled(scale)
                .positioned(subpixel.into());
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
