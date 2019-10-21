use crate::text::*;

pub struct DummyProvider;

impl FontProvider for DummyProvider {
    fn get_glyph_id(&self, c: char) -> u32 {
        c as u32
    }

    fn get_line_height(&self, font_size: f32) -> f32 {
        font_size + 1.0
    }

    fn get_advance(&self, _from: u32, _to: u32, font_size: f32) -> Option<f32> {
        Some(font_size / 2.0)
    }

    fn get_metric(&self, _id: u32, font_size: f32) -> RectPx {
        RectPx {
            x: 0.0,
            y: font_size * 0.66 / 3.0,
            w: font_size / 2.0,
            h: font_size * 2.0 / 3.0,
        }
    }

    fn render_glyph(&mut self, id: u32, _font_size: f32) -> Option<RectUv> {
        use std::convert::TryFrom;
        let c = char::try_from(id).ok()?;
        if c.is_whitespace() {
            None
        } else {
            Some(RectUv {
                x: -1.0,
                y: -1.0,
                w: 0.0,
                h: 0.0,
            })
        }
    }

    fn update_glyph_cache_expiration(&mut self) {}
}
