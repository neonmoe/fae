//! For manually setting the visual layout of elements.
//!
//! # Examples
//!
//! ## Setting layout manually from code
//! ```
//! use fungui::{layout, element};
//!
//! layout::push_padding(20.0);
//! layout::push_direction(layout::Direction::Right);
//! layout::push_rect(
//!     layout::screen_x(0.5) - 100.0,
//!     20.0,
//!     200.0,
//!     16.0,
//! );
//!
//! for i in 0..10 {
//!     element::label(&format!("button-{}", i), &format!("Button #{}", i));
//! }
//!
//! // Each push (eg. push_rect) has a corresponding pop
//! // (eg. pop_rect). The pops are technically not needed, but the
//! // intention is to build a sort of "style stack" and pop as your
//! // environment changes.
//! layout::pop_rect();
//! layout::pop_direction();
//! layout::pop_padding();
//! ```
use super::*;
pub use text::Alignment;

lazy_static! {
    static ref LAYOUT_CONTROLLER: Mutex<LayoutController> = Mutex::new(LayoutController {
        rect: vec![Rect {
            left: 32.0,
            top: 32.0,
            right: 132.0,
            bottom: 48.0
        }],
        padding: vec![32.0],
        direction: vec![Direction::Down],
        alignment: vec![Alignment::Center],
    });
}

struct LayoutController {
    rect: Vec<Rect>,
    padding: Vec<f32>,
    direction: Vec<Direction>,
    alignment: Vec<Alignment>,
}

/// Pushes a rectangle on the rectangle stack.
///
/// To return to the rectangle used before, use `pop_rect`.
pub fn push_rect(x: f32, y: f32, width: f32, height: f32) {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    lock.rect.push(Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    });
}

/// Pushes a direction on the direction stack.
///
/// To return to the direction used before, use `pop_direction`.
pub fn push_direction(direction: Direction) {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    lock.direction.push(direction);
}

/// Pushes a padding value on the padding value stack.
///
/// To return to the padding used before, use `pop_padding`.
pub fn push_padding(padding: f32) {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    lock.padding.push(padding);
}

/// Pushes an alignment on the alignment stack.
///
/// To return to the alignment used before, use `pop_alignment`.
pub fn push_alignment(alignment: Alignment) {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    lock.alignment.push(alignment);
}

/// Pops a rectangle off the rectangle stack.
pub fn pop_rect() {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    if lock.rect.len() > 1 {
        lock.rect.pop();
    }
}

/// Pops a direction off the direction stack.
pub fn pop_direction() {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    if lock.direction.len() > 1 {
        lock.direction.pop();
    }
}

/// Pops a padding value off the padding value stack.
pub fn pop_padding() {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    if lock.padding.len() > 1 {
        lock.padding.pop();
    }
}

/// Pops an alignment off the alignment stack.
pub fn pop_alignment() {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    if lock.alignment.len() > 1 {
        lock.alignment.pop();
    }
}

/// Returns `x` of window width, ie. `screen_x(0.5)` with a `640x480`
/// screen will return `320.0` (= `640 * 0.5`).
pub fn screen_x(x: f32) -> f32 {
    let lock = WINDOW_DIMENSIONS.lock().unwrap();
    x * lock.0
}

/// Returns `y` of window height, ie. `screen_y(0.5)` with a `640x480`
/// screen will return `240.0` (= `480 * 0.5`).
pub fn screen_y(y: f32) -> f32 {
    let lock = WINDOW_DIMENSIONS.lock().unwrap();
    y * lock.1
}

pub(crate) fn create_next_element() -> (Rect, Alignment) {
    let mut lock = LAYOUT_CONTROLLER.lock().unwrap();
    let current_rect = *lock.rect.last().unwrap();
    let alignment = *lock.alignment.last().unwrap();

    let padding = *lock.padding.last().unwrap();
    let direction = *lock.direction.last().unwrap();
    let rect = lock.rect.last_mut().unwrap();
    let offset = (
        current_rect.width() + padding,
        current_rect.height() + padding,
    );
    match direction {
        Direction::Left => *rect += (-offset.0, 0.0),
        Direction::Up => *rect += (0.0, -offset.1),
        Direction::Right => *rect += (offset.0, 0.0),
        Direction::Down => *rect += (0.0, offset.1),
    }

    (current_rect, alignment)
}

/// Defines a rectangle by defining each side's position on its
/// axis.
///
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

impl std::ops::AddAssign<(f32, f32)> for Rect {
    fn add_assign(&mut self, other: (f32, f32)) {
        self.left += other.0;
        self.top += other.1;
        self.right += other.0;
        self.bottom += other.1;
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
