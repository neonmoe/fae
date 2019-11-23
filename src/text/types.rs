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
pub struct Glyph {
    pub id: GlyphId,
    pub cursor: Cursor,
    pub metrics: RectPx,
    pub draw_data: usize,
}

#[derive(Clone, Copy)]
pub struct TextDrawData {
    pub clip_area: Option<Rect>,
    pub color: (f32, f32, f32, f32),
    pub font_size: i32,
    pub z: f32,
}

pub trait FontProvider {
    fn get_glyph_id(&mut self, c: char) -> Option<GlyphId>;
    fn get_line_advance(&self, cursor: Cursor, font_size: i32) -> Advance;
    fn get_advance(
        &mut self,
        from: GlyphId,
        to: GlyphId,
        cursor: Cursor,
        font_size: i32,
    ) -> Advance;
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
    font_size: Option<i32>,
}

impl CacheIdentifier {
    pub fn new(id: GlyphId, font_size: Option<i32>) -> CacheIdentifier {
        CacheIdentifier { id, font_size }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    pub x: i32,
    pub y: i32,
    // The fractional parts that should be "consumed" by whitespace
    // Generally: accumulated width should be added to a space (' ')
    // glyph's advance, and then set to 0.
    // TODO: Consider ripping out the space accumulator, if it isn't made into an option
    pub space_accumulator: f32,
}

impl Cursor {
    pub fn new(x: i32, y: i32) -> Cursor {
        Cursor {
            x,
            y,
            space_accumulator: 0.0,
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

#[derive(Clone, Copy, Debug)]
pub struct Advance {
    pub advance_x: i32,
    pub advance_y: i32,
    pub space_accumulator: f32,
}

impl From<Cursor> for Advance {
    fn from(other: Cursor) -> Advance {
        Advance {
            advance_x: 0,
            advance_y: 0,
            space_accumulator: other.space_accumulator,
        }
    }
}

impl Add<Advance> for Cursor {
    type Output = Cursor;
    fn add(mut self, other: Advance) -> Self::Output {
        self.x += other.advance_x;
        self.y += other.advance_y;
        self.space_accumulator = other.space_accumulator;
        self
    }
}
