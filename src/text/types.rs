use crate::error::GlyphNotRenderedError;
use crate::renderer::Renderer;
use crate::text::GlyphCache;
use crate::types::*;

use std::ops::Add;

pub(crate) type GlyphId = u32;

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
pub(crate) struct Glyph {
    pub id: GlyphId,
    pub cursor: Cursor,
    pub metric: RectPx,
    pub draw_data: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct TextDrawData {
    pub position: (f32, f32),
    pub clip_area: Option<Rect>,
    pub color: (f32, f32, f32, f32),
    pub rotation: (f32, f32, f32),
    pub font_size: i32,
    pub z: f32,
}

pub(crate) trait FontProvider {
    fn get_glyph_id(&mut self, c: char) -> GlyphId;
    fn get_line_advance(&self, font_size: i32) -> Advance;
    fn get_advance(&mut self, from: GlyphId, to: GlyphId, font_size: i32) -> Advance;
    fn get_metric(&mut self, id: GlyphId, font_size: i32) -> RectPx;
    fn render_glyph(
        &mut self,
        renderer: &mut Renderer,
        cache: &mut GlyphCache,
        id: GlyphId,
        font_size: i32,
    ) -> Result<RectPx, GlyphNotRenderedError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CacheIdentifier {
    id: GlyphId,
    font_size: Option<i32>,
}

impl CacheIdentifier {
    pub fn new(id: GlyphId, font_size: Option<i32>) -> CacheIdentifier {
        CacheIdentifier { id, font_size }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor {
    pub x: i32,
    pub y: i32,
}

impl Cursor {
    pub fn new(x: i32, y: i32) -> Cursor {
        Cursor { x, y }
    }
}

impl Add<Cursor> for RectPx {
    type Output = RectPx;
    fn add(mut self, other: Cursor) -> Self::Output {
        self.x += other.x;
        self.y += other.y;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Advance {
    pub advance_x: i32,
    pub advance_y: i32,
}

impl Add<Advance> for Cursor {
    type Output = Cursor;
    fn add(mut self, other: Advance) -> Self::Output {
        self.x += other.advance_x;
        self.y += other.advance_y;
        self
    }
}
