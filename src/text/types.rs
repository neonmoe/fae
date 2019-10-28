use crate::types::*;

/// Defines the alignment of text.
#[derive(Clone, Copy, Debug)]
pub enum Alignment {
    /// Text is aligned to the left.
    Left,
    /// Text is aligned to the right.
    Right,
    /// Text is centered.
    Center,
}

#[derive(Clone, Copy)]
pub(crate) struct Metric {
    pub glyph_id: u32,
    pub size: RectPx,
}

#[derive(Clone, Copy)]
pub(crate) struct Glyph {
    pub screen_location: RectPx,
    pub id: u32,
    pub draw_data: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct TextDrawData {
    pub clip_area: Option<Rect>,
    pub color: (f32, f32, f32, f32),
    pub font_size: f32,
    pub z: f32,
}

pub(crate) trait FontProvider {
    fn get_glyph_id(&self, c: char) -> u32;
    fn get_line_height(&self, font_size: f32) -> i32;
    fn get_advance(&self, from: u32, to: u32, font_size: f32) -> Option<i32>;
    fn get_metric(&self, id: u32, font_size: f32) -> RectPx;
    fn render_glyph(&mut self, id: u32, font_size: f32) -> Option<RectPx>;
    fn update_glyph_cache_expiration(&mut self);
}
