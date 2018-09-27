//! For manually setting the visual layout of elements.
//!
//! # Examples
//!
//! ## Setting layout manually from code
//! ```
//! use fungui::element::button;
//! use fungui::layout::{define_element_layout, Alignment, UIElementLayout};
//!
//! // Before your main loop
//! define_element_layout(
//!     "big_button",
//!     UIElementLayout::new()
//!         .anchors(0.0, 0.0, 1.0, 1.0)
//!         .alignment(Alignment::Center)
//! );
//!
//! /* ... */
//!
//! // Inside your main loop
//! if button("big_button", "Very Big Button") {
//!     println!("Clicked the big button.");
//! }
//! ```
pub use super::define_element_layout;
use super::*;
pub use text::Alignment;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Rect {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }
}

/// Contains the visual properties of an element. Used with
/// `define_element_layout`.
///
/// [See the `layout` documentation for an
/// example](index.html#setting-layout-manually-from-code).
#[derive(Clone, Copy, Debug)]
pub struct UIElementLayout {
    pub(crate) relative: Rect,
    pub(crate) anchors: Rect,
    pub(crate) alignment: Alignment,
}

impl Default for UIElementLayout {
    fn default() -> UIElementLayout {
        UIElementLayout {
            relative: Rect {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            anchors: Rect {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            alignment: Alignment::Center,
        }
    }
}

impl UIElementLayout {
    /// Creates a new `UIElementLayout`.
    ///
    /// Default `relative` and `anchors` are zeroed, and `alignment`
    /// is `Alignment::Center`.
    pub fn new() -> UIElementLayout {
        UIElementLayout {
            ..Default::default()
        }
    }

    /// Sets the relative coordinates of the element.
    ///
    /// The values are in logical pixel coordinates (that is,
    /// DPI-aware pixels). They are added to the origin defined by
    /// `anchors` when calculating on-screen dimensions.
    pub fn relative(mut self, left: f32, top: f32, right: f32, bottom: f32) -> UIElementLayout {
        self.relative = Rect {
            left,
            top,
            right,
            bottom,
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
    pub fn anchors(mut self, left: f32, top: f32, right: f32, bottom: f32) -> UIElementLayout {
        self.anchors = Rect {
            left,
            top,
            right,
            bottom,
        };
        self
    }

    /// Sets the alignment of the text inside the element.
    pub fn alignment(mut self, alignment: Alignment) -> UIElementLayout {
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
