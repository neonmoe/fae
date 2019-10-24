// Layout

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

// Geometry
use std::ops::{Add, AddAssign};

#[derive(Clone, Copy)]
pub(crate) struct RectUv {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct RectPx<T: Add + AddAssign> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

#[derive(Clone, Copy)]
pub(crate) struct PositionPx<T: Add + AddAssign> {
    pub x: T,
    pub y: T,
}

impl<T: Add + AddAssign> Add<PositionPx<T>> for RectPx<T> {
    type Output = RectPx<T>;
    fn add(mut self, other: PositionPx<T>) -> Self::Output {
        self.x += other.x;
        self.y += other.y;
        self
    }
}

// Fonts

#[derive(Clone, Copy)]
pub(crate) struct Metric {
    pub glyph_id: u32,
    pub size: RectPx<f32>,
}

#[derive(Clone, Copy)]
pub(crate) struct Glyph {
    pub screen_location: RectPx<f32>,
    pub id: u32,
    pub draw_data: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct TextDrawData {
    pub clip_area: Option<(f32, f32, f32, f32)>,
    pub color: (f32, f32, f32, f32),
    pub font_size: f32,
    pub z: f32,
}

pub(crate) trait FontProvider {
    fn get_glyph_id(&self, c: char) -> u32;
    fn get_line_height(&self, font_size: f32) -> f32;
    fn get_advance(&self, from: u32, to: u32, font_size: f32) -> Option<f32>;
    fn get_metric(&self, id: u32, font_size: f32) -> RectPx<f32>;
    fn render_glyph(&mut self, id: u32, font_size: f32) -> Option<RectUv>;
    fn update_glyph_cache_expiration(&mut self);
}
