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

#[derive(Clone, Copy)]
pub(crate) struct RectUv {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct RectPx {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct PositionPx {
    pub x: f32,
    pub y: f32,
}

impl std::ops::Add<PositionPx> for RectPx {
    type Output = RectPx;
    fn add(mut self, other: PositionPx) -> Self::Output {
        self.x += other.x;
        self.y += other.y;
        self
    }
}

// Fonts

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
    pub clip_area: Option<(f32, f32, f32, f32)>,
    pub color: (f32, f32, f32, f32),
    pub font_size: f32,
    pub z: f32,
}

pub(crate) trait FontProvider {
    fn get_glyph_id(&self, c: char) -> u32;
    fn get_line_height(&self, font_size: f32) -> f32;
    fn get_advance(&self, from: u32, to: u32, font_size: f32) -> Option<f32>;
    fn get_metric(&self, id: u32, font_size: f32) -> RectPx;
    fn render_glyph(&mut self, id: u32, font_size: f32) -> Option<RectUv>;
    fn update_glyph_cache_expiration(&mut self);
}
