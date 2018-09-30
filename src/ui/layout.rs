//! For manually setting the visual layout of elements.
//!
//! # Examples
//!
//! ## Setting layout manually from code
//! ```
// TODO: Write this
//! ```
use super::*;
pub use text::Alignment;

lazy_static! {
    static ref LAYOUT_CONTROLLER: Mutex<LayoutController> = Mutex::new(LayoutController {
        layout: Layout::new().relative(16.0, 16.0, 116.0, 32.0),
        direction: Direction::Down,
    });
}

struct LayoutController {
    layout: Layout,
    direction: Direction,
}

/// Defines the next element's layout. The elements following that
/// will use the same layout as well, offset in the direction
/// specified by `define_direction`.
pub fn define_layout(layout: Layout) {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    lock.layout = layout;
}

/// Defines the direction in which the next element is offset from the
/// last one.
pub fn define_direction(direction: Direction) {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    lock.direction = direction;
}

/// Defines a rectangle by defining each side's position on its
/// axis. Ie. a 1x1 rectangle that was 2 units offset on the x-axis,
/// would be defined as:
/// ```
/// # use fungui::layout::Rect;
/// Rect { left: 3.0, top: 1.0, right: 4.0, bottom: 0.0 };
/// ```
/// The crate considers positive x as right, positive y as down.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Rect {
    /// Distance of the rectangle's left side from the origin on the
    /// x-axis.
    pub left: f32,
    /// Distance of the rectangle's top side from the origin on the
    /// y-axis.
    pub top: f32,
    /// Distance of the rectangle's right side from the origin on the
    /// x-axis.
    pub right: f32,
    /// Distance of the rectangle's bottom side from the origin on the
    /// y-axis.
    pub bottom: f32,
}

impl Rect {
    /// Returns the width of the rectangle.
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    /// Returns the height of the rectangle.
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }
}

/// Describes a direction.
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    /// Backwards on the x-axis.
    Left,
    /// Backwards on the y-axis.
    Up,
    /// Forwards on the x-axis.
    Right,
    /// Forwards on the y-axis.
    Down,
}

// TODO: Layout instances -> style pushing / popping
// eg. no "define_layout(Layout::new().relative(..).anchors(..));"
// replace with "push_relative(..); push_anchors(..);"
// This is supposed to be immediate-mode-like after all.

/// Contains the visual properties of an element. Used with
/// `define_element_layout`.
///
/// [See the `layout` documentation for an
/// example](index.html#setting-layout-manually-from-code).
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub(crate) relative: Rect,
    pub(crate) anchors: Rect,
    pub(crate) alignment: Alignment,
    padding: f32,
}

impl Default for Layout {
    fn default() -> Layout {
        Layout {
            relative: Rect {
                left: 8.0,
                top: 8.0,
                right: 108.0,
                bottom: 24.0,
            },
            anchors: Rect {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            alignment: Alignment::Center,
            padding: 8.0,
        }
    }
}

impl Layout {
    /// Creates a new `Layout`.
    ///
    /// Default `relative` and `anchors` are zeroed, and `alignment`
    /// is `Alignment::Center`.
    pub fn new() -> Layout {
        Layout {
            ..Default::default()
        }
    }

    pub(crate) fn for_next_element() -> Layout {
        let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
        let current_layout = lock.layout;
        let mut new_layout = current_layout;
        match lock.direction {
            Direction::Left => {
                let w = new_layout.relative.width() + new_layout.padding;
                new_layout.move_relative(-w, 0.0, -w, 0.0);
            }

            Direction::Up => {
                let h = new_layout.relative.height() + new_layout.padding;
                new_layout.move_relative(0.0, -h, 0.0, -h);
            }

            Direction::Right => {
                let w = new_layout.relative.width() + new_layout.padding;
                new_layout.move_relative(w, 0.0, w, 0.0);
            }

            Direction::Down => {
                let h = new_layout.relative.height() + new_layout.padding;
                new_layout.move_relative(0.0, h, 0.0, h);
            }
        }
        lock.layout = new_layout;
        current_layout
    }

    /// Sets the relative coordinates of the element.
    ///
    /// The values are in logical pixel coordinates (that is,
    /// DPI-aware pixels). They are added to the origin defined by
    /// `anchors` when calculating on-screen dimensions.
    pub fn relative<F: Into<f32>>(mut self, left: F, top: F, right: F, bottom: F) -> Layout {
        self.relative = Rect {
            left: left.into(),
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
        };
        self
    }

    /// Sets the anchors of the element.
    ///
    /// The anchors act as a way to scale elements with the
    /// window. Each value is multiplied by the window dimension, and
    /// added to the corresponding `relative` value when calculating
    /// the final layout of the element. To illustrate, the final
    /// 'left' is calculated in the following way: `on_screen.left =
    /// relative.left + anchors.left * WINDOW_WIDTH`.
    pub fn anchors<F: Into<f32>>(mut self, left: F, top: F, right: F, bottom: F) -> Layout {
        self.anchors = Rect {
            left: left.into(),
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
        };
        self
    }

    /// Sets the padding of the element.
    ///
    /// This is the distance between two consecutive elements.
    pub fn padding<F: Into<f32>>(mut self, d: F) -> Layout {
        self.padding = d.into();
        self
    }

    /// Appends the given coordinates to the current relative
    /// coordinates. See the `relative()` docs for what the relative
    /// coordinates are.
    pub fn move_relative<F: Into<f32>>(&mut self, left: F, top: F, right: F, bottom: F) {
        self.relative.left += left.into();
        self.relative.top += top.into();
        self.relative.right += right.into();
        self.relative.bottom += bottom.into();
    }

    /// Appends the given coordinates to the current anchor
    /// coordinates. See the `anchors()` docs for what the anchor
    /// coordinates are.
    pub fn move_anchors<F: Into<f32>>(&mut self, left: F, top: F, right: F, bottom: F) {
        self.anchors.left += left.into();
        self.anchors.top += top.into();
        self.anchors.right += right.into();
        self.anchors.bottom += bottom.into();
    }

    /// Sets the alignment of the text inside the element.
    pub fn alignment(mut self, alignment: Alignment) -> Layout {
        self.alignment = alignment;
        self
    }

    pub(crate) fn absolute(&self) -> Rect {
        let lock = WINDOW_DIMENSIONS.lock().unwrap();
        let (width, height) = *lock;
        Rect {
            left: self.relative.left + width * self.anchors.left,
            top: self.relative.top + height * self.anchors.top,
            right: self.relative.right + width * self.anchors.right,
            bottom: self.relative.bottom + height * self.anchors.bottom,
        }
    }
}
