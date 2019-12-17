//! Types used throughout the crate.

/// Represents a rectangle.
///
/// # Usage
/// ```
/// # use fae::Rect;
/// // Rects can be defined via the struct
/// let rect = Rect { x: 1.0, y: 1.0, width: 5.0, height: 5.0 };
/// // And via a tuple as well (x, y, width, height):
/// let rect_ = (1.0, 1.0, 5.0, 5.0).into();
/// assert_eq!(rect, rect_);
/// ```
///
/// Tip: Many functions in `fae` take `Into<Rect>` as a parameter, in
/// which case it is often cleaner to pass in a tuple, like `rect_`
/// above, but without the `.into()`.
// Note: Only axis-aligned rectangles are allowed (rotation is
// specified via another parameter) because this is much more
// optimizable, and I don't intend to support
// non-axis-aligned-rectangles.
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

impl From<(i32, i32, i32, i32)> for Rect {
    fn from(from: (i32, i32, i32, i32)) -> Self {
        Rect {
            x: from.0 as f32,
            y: from.1 as f32,
            width: from.2 as f32,
            height: from.3 as f32,
        }
    }
}

impl From<RectPx> for Rect {
    fn from(from: RectPx) -> Self {
        Rect {
            x: from.x as f32,
            y: from.y as f32,
            width: from.width as f32,
            height: from.height as f32,
        }
    }
}

/// Like Rect, but i32-based. Internal use only, at least currently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RectPx {
    /// The x-coordinate of the top-left corner of this rectangle.
    pub x: i32,
    /// The y-coordinate of the top-left corner of this rectangle.
    pub y: i32,
    /// The width of this rectangle.
    pub width: i32,
    /// The height of this rectangle.
    pub height: i32,
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

impl From<RectPx> for (i32, i32, i32, i32) {
    fn from(from: RectPx) -> Self {
        (from.x, from.y, from.width, from.height)
    }
}
