//! This is a simple implementation of a rectangle. Can be defined
//! with top-left coordinates and dimensions, or top-left and
//! bottom-right coordinates.

/// Represents a rectangle.
#[derive(Clone, Copy, Debug)]
pub enum Rect {
    /// This representation consists of the rectangle's top-left
    /// corner's coordinates and its dimensions in the order: `left x,
    /// top y, width, height`.
    Dims(f32, f32, f32, f32),
    /// This representation consists of the top-left and bottom-right
    /// coordinates in the order: `left x, top y, right x, bottom y`.
    Coords(f32, f32, f32, f32),
}

impl Rect {
    /// Returns the rectangle's top-left and bottom-right coordinates
    /// in the same order as `Rect::Coords`.
    pub fn coords(&self) -> (f32, f32, f32, f32) {
        (self.left(), self.top(), self.right(), self.bottom())
    }

    /// Returns the rectangle's top-left corner and dimensions in the
    /// same order as `Rect::Dims`.
    pub fn dims(&self) -> (f32, f32, f32, f32) {
        (self.left(), self.top(), self.width(), self.height())
    }

    /// Returns the x coordinate of the rectangle's top-left corner.
    pub fn left(&self) -> f32 {
        match *self {
            Rect::Dims(x, ..) => x,
            Rect::Coords(x, ..) => x,
        }
    }

    /// Returns the y coordinate of the rectangle's top-left corner.
    pub fn top(&self) -> f32 {
        match *self {
            Rect::Dims(_, y, ..) => y,
            Rect::Coords(_, y, ..) => y,
        }
    }

    /// Returns the x coordinate of the rectangle's bottom-right corner.
    pub fn right(&self) -> f32 {
        match *self {
            Rect::Dims(x, _, w, _) => x + w,
            Rect::Coords(.., x1, _) => x1,
        }
    }

    /// Returns the y coordinate of the rectangle's bottom-right corner.
    pub fn bottom(&self) -> f32 {
        match *self {
            Rect::Dims(_, y, _, h) => y + h,
            Rect::Coords(.., y1) => y1,
        }
    }

    /// Returns the width of the rectangle.
    pub fn width(&self) -> f32 {
        match *self {
            Rect::Dims(.., w, _) => w,
            Rect::Coords(x0, _, x1, ..) => x1 - x0,
        }
    }

    /// Returns the height of the rectangle.
    pub fn height(&self) -> f32 {
        match *self {
            Rect::Dims(.., h) => h,
            Rect::Coords(_, y0, _, y1) => y1 - y0,
        }
    }

    /// Returns the width and height of the image in a tuple.
    pub fn dimensions(&self) -> (f32, f32) {
        match *self {
            Rect::Dims(.., w, h) => (w, h),
            Rect::Coords(x0, y0, x1, y1) => (x1 - x0, y1 - y0),
        }
    }

    /// Sets the rectangle's top-left and bottom-right coordinates
    /// in the same order as `Rect::Coords`.
    pub fn set_coords(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        *self = Rect::Coords(x0, y0, x1, y1);
    }

    /// Sets the rectangle's top-left coordinates and dimensions in
    /// the same order as `Rect::Dims`.
    pub fn set_dims(&mut self, x: f32, y: f32, width: f32, height: f32) {
        *self = Rect::Dims(x, y, width, height);
    }
}
