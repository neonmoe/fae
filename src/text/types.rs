use crate::error::GlyphNotRenderedError;
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
    pub metrics: RectPx,
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
    fn get_line_advance(&self, cursor: Cursor, font_size: i32) -> Advance;
    fn get_advance(&self, from: GlyphId, to: GlyphId, cursor: Cursor, font_size: i32) -> Advance;
    fn get_metric(&mut self, id: GlyphId, cursor: Cursor, font_size: i32) -> RectPx;
    fn render_glyph(
        &mut self,
        cache: &mut GlyphCache,
        id: GlyphId,
        cursor: Cursor,
        font_size: i32,
    ) -> Result<RectPx, GlyphNotRenderedError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CacheIdentifier {
    id: GlyphId,
    font_size: i32,
}

impl CacheIdentifier {
    pub fn new(id: GlyphId, font_size: i32) -> CacheIdentifier {
        CacheIdentifier { id, font_size }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Advance {
    pub x: i32,
    pub y: i32,
    pub leftover_x: f32,
    pub leftover_y: f32,
}

impl Advance {
    pub fn new(x: i32, y: i32, leftover_x: f32, leftover_y: f32) -> Advance {
        Advance {
            x,
            y,
            leftover_x,
            leftover_y,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor {
    pub x: i32,
    pub y: i32,
    pub leftover_x: f32,
    pub leftover_y: f32,
}

impl Cursor {
    pub fn new(x: i32, y: i32) -> Cursor {
        Cursor {
            x,
            y,
            leftover_x: 0.0,
            leftover_y: 0.0,
        }
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

impl Add<Advance> for Cursor {
    type Output = Cursor;
    fn add(mut self, other: Advance) -> Self::Output {
        self.x += other.x;
        self.y += other.y;
        self.leftover_x = other.leftover_x;
        self.leftover_y = other.leftover_y;
        self
    }
}
