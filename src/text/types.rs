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
    font_size: Option<i32>,
    subpixel: Option<SubpixelOffset>,
}

impl CacheIdentifier {
    pub fn new(
        id: GlyphId,
        font_size: Option<i32>,
        subpixel: Option<SubpixelOffset>,
    ) -> CacheIdentifier {
        CacheIdentifier {
            id,
            font_size,
            subpixel,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    pub x: i32,
    pub y: i32,
    // The fractional parts leftover by previous advances
    pub leftover_x: f32,
    pub leftover_y: f32,
    // The fractional parts that should be "consumed" by whitespace
    // Generally: accumulated width should be added to a space (' ')
    // glyph's advance, and then set to 0.
    // TODO: This should be an option of the text renderer, in addition to line height and distance between characters
    // ^ Or alternatively it would be good to actually handle the fractions where they happen. Maybe an option?
    pub space_accumulator: f32,
}

impl Cursor {
    pub fn new(x: i32, y: i32) -> Cursor {
        Cursor {
            x,
            y,
            leftover_x: 0.0,
            leftover_y: 0.0,
            space_accumulator: 0.0,
        }
    }

    pub fn subpixel_offset(self) -> SubpixelOffset {
        SubpixelOffset {
            x: (self.leftover_x * SUBPIXEL_RESOLUTION) as i32,
            y: (self.leftover_y * SUBPIXEL_RESOLUTION) as i32,
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
    pub leftover_x: f32,
    pub leftover_y: f32,
}

impl From<Cursor> for Advance {
    fn from(other: Cursor) -> Advance {
        Advance {
            advance_x: 0,
            advance_y: 0,
            space_accumulator: other.space_accumulator,
            leftover_x: other.leftover_x,
            leftover_y: other.leftover_y,
        }
    }
}

impl Add<Advance> for Cursor {
    type Output = Cursor;
    fn add(mut self, other: Advance) -> Self::Output {
        self.x += other.advance_x;
        self.y += other.advance_y;
        self.space_accumulator = other.space_accumulator;
        self.leftover_x = other.leftover_x;
        self.leftover_y = other.leftover_y;
        self
    }
}

// TODO: Making subpixel offset granularity a runtime option might be good
const SUBPIXEL_RESOLUTION: f32 = 4.0;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubpixelOffset {
    x: i32,
    y: i32,
}

#[cfg(feature = "rusttype")]
impl From<SubpixelOffset> for rusttype::Point<f32> {
    fn from(src: SubpixelOffset) -> Self {
        rusttype::point(
            src.x as f32 / SUBPIXEL_RESOLUTION,
            src.y as f32 / SUBPIXEL_RESOLUTION,
        )
    }
}
