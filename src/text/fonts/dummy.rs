use crate::text::*;

pub struct DummyProvider;

fn get_size(font_size: f32) -> i32 {
    (font_size / 16.0).max(1.0).round() as i32 * 8
}

impl FontProvider for DummyProvider {
    fn get_glyph_id(&self, c: char) -> u32 {
        c as u32
    }

    fn get_line_height(&self, font_size: f32) -> i32 {
        get_size(font_size) * 4 / 3
    }

    fn get_advance(&self, _from: u32, _to: u32, font_size: f32) -> Option<i32> {
        let size = get_size(font_size);
        Some(size + 1)
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

    fn render_glyph(&mut self, _id: u32, _font_size: f32) -> Option<RectPx> {
        Some(RectPx {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        })
    }

    fn update_glyph_cache_expiration(&mut self) {}
}
