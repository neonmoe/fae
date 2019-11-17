use crate::error::GlyphNotRenderedError;
use crate::text::GlyphCache;
use crate::types::*;

pub(crate) type GlyphId = u16;

/// Defines the alignment of text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    pub glyph_id: GlyphId,
    pub size: RectPx,
}

#[derive(Clone, Copy)]
pub(crate) struct Glyph {
    pub screen_location: RectPx,
    pub id: GlyphId,
    pub draw_data: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct TextDrawData {
    pub clip_area: Option<Rect>,
    pub color: (f32, f32, f32, f32),
    pub font_size: i32,
    pub z: f32,
}

pub(crate) trait FontProvider {
    fn get_glyph_id(&self, c: char) -> Option<GlyphId>;
    fn get_line_height(&self, font_size: i32) -> i32;
    fn get_advance(&self, from: GlyphId, to: GlyphId, font_size: i32) -> Option<i32>;
    fn get_metric(&self, id: GlyphId, font_size: i32) -> RectPx;
    fn render_glyph(
        &mut self,
        cache: &mut GlyphCache,
        id: GlyphId,
        font_size: i32,
    ) -> Result<RectPx, GlyphNotRenderedError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CacheIdentifier {
    id: u16,
    font_size: i32,
}

impl CacheIdentifier {
    pub fn new(id: u16, font_size: i32) -> CacheIdentifier {
        CacheIdentifier { id, font_size }
    }
}
