use std::ops::Add;

/// Represents a rectangle.
///
/// # Usage
/// ```
/// # use fae::Rect;
/// // Rects can be defined via the struct
/// let rect = Rect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 };
/// // And via a tuple as well (x, y, width, height):
/// let rect_ = (0.0, 0.0, 1.0, 1.0).into();
/// assert_eq!(rect, rect_);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// The x-coordinate of the top-left corner of this rectangle.
    pub x: f32,
    /// The y-coordinate of the top-left corner of this rectangle.
    pub y: f32,
    /// The width of this rectangle.
    pub width: f32,
    /// The height of this rectangle.
    pub height: f32,
}

impl Rect {
    pub(crate) fn into_corners(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + self.width, self.y + self.height)
    }
}

impl From<(f32, f32, f32, f32)> for Rect {
    fn from(from: (f32, f32, f32, f32)) -> Self {
        Rect {
            x: from.0,
            y: from.1,
            width: from.2,
            height: from.3,
        }
    }
}

/// Represents a rectangle in a coordinate space that consists of
/// integers.
///
/// # Usage
/// ```
/// # use fae::RectPx;
/// // RectPxs can be defined via the struct
/// let rect = RectPx { x: 0, y: 0, width: 16, height: 16 };
/// // And via a tuple as well (x, y, width, height):
/// let rect_ = (0, 0, 16, 16).into();
/// assert_eq!(rect, rect_);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RectPx {
    /// The x-coordinate of the top-left corner of this rectangle.
    pub x: i32,
    /// The y-coordinate of the top-left corner of this rectangle.
    pub y: i32,
    /// The width of this rectangle.
    pub width: i32,
    /// The height of this rectangle.
    pub height: i32,
}

impl RectPx {
    pub(crate) fn into_corners(self) -> (f32, f32, f32, f32) {
        (
            self.x as f32,
            self.y as f32,
            (self.x + self.width) as f32,
            (self.y + self.height) as f32,
        )
    }
}

impl From<(i32, i32, i32, i32)> for RectPx {
    fn from(from: (i32, i32, i32, i32)) -> Self {
        RectPx {
            x: from.0,
            y: from.1,
            width: from.2,
            height: from.3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PositionPx {
    pub x: i32,
    pub y: i32,
}

impl From<(i32, i32)> for PositionPx {
    fn from(from: (i32, i32)) -> Self {
        PositionPx {
            x: from.0,
            y: from.1,
        }
    }
}

impl Add<PositionPx> for RectPx {
    type Output = RectPx;
    fn add(mut self, other: PositionPx) -> Self::Output {
        self.x += other.x;
        self.y += other.y;
        self
    }
}