//! Contains the functions that create the UI elements.
mod input;
pub(crate) use self::input::*;

use rect::Rect;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use text::Alignment;
use ui::{UIState, PADDING};

/// Represents what kind of element this is, used for deciding which
/// sprite to use for rendering, and which state the element is in.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UIElementKind {
    /// Element without a background graphic.
    NoBackground = -1,
    /// Unpressed regular button.
    ButtonNormal = 0,
    /// Button which is being hovered.
    ButtonHovered,
    /// Button which is being pressed down.
    ButtonPressed,
    /// Editable text field.
    InputField,
    /// How many different UI element kinds there are.
    KindCount,
}

/// Contains data needed to render an UI element, as well as the state
/// of the element.
#[derive(Clone, Debug)]
pub struct UIElement {
    /// A unique identifier for the element, used for persistent
    /// state.
    pub identifier: String,
    /// The kind of the element.
    // TODO: Decouple element state from this field
    pub kind: UIElementKind,
    /// The element's coordinates and dimensions on the screen.
    pub rect: Rect,
    /// The alignment of the element's text.
    pub alignment: Alignment,
}

impl UIElement {
    pub fn create(identifier: String, kind: UIElementKind) -> UIElement {
        UIElement {
            identifier,
            kind,
            rect: Rect::Dims(10.0, 10.0, 150.0, 16.0),
            alignment: Alignment::Left,
        }
    }

    pub(crate) fn id(&self) -> u64 {
        element_hash(&self.identifier)
    }

    pub(crate) fn is_point_inside(&self, x: f32, y: f32) -> bool {
        let (x0, y0, x1, y1) = self.rect.coords();
        !(x < x0 - PADDING || x >= x1 + PADDING || y < y0 - PADDING || y >= y1 + PADDING)
    }
}

pub(crate) fn element_hash(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

impl UIState {
    /// Creates a text label. Used for displaying plain uneditable text.
    pub fn label(&mut self, identifier: &str, display_text: &str) {
        let element = UIElement::create(identifier.to_owned(), UIElementKind::NoBackground);
        self.draw_element(&element, display_text, true, None);
        self.insert_element(element);
    }

    /// Renders a button that can be pressed, and returns whether or not
    /// it was clicked.
    pub fn button(&mut self, identifier: &str, display_text: &str) -> bool {
        let (clicked, element) = button_meta(self, identifier);
        self.draw_element(&element, display_text, false, None);
        self.insert_element(element);
        clicked
    }

    /// Renders a button that can be pressed, and returns whether or not
    /// it was clicked. In contrast to `button`, this version lets you
    /// define the sprite used yourself. That said, the size and position
    /// of the button is still controlled by the layout system.
    pub fn button_image(
        &mut self,
        identifier: &str,
        texcoords: (f32, f32, f32, f32),
        color: (u8, u8, u8, u8),
        z: f32,
        tex_index: usize,
    ) -> bool {
        let (clicked, element) = button_meta(self, identifier);
        self.renderer
            .draw_quad(element.rect.coords(), texcoords, color, z, tex_index);
        self.insert_element(element);
        clicked
    }
}

// TODO: Implement button ordering and only activate one button per press
fn button_meta(ui: &mut UIState, identifier: &str) -> (bool, UIElement) {
    let mut element = UIElement::create(identifier.to_owned(), UIElementKind::ButtonNormal);
    let id = element.id();
    let hovered = element.is_point_inside(ui.mouse.x, ui.mouse.y);
    let just_released = ui.mouse.clicked();
    let can_be_pressed =
        ui.pressed_element.is_none() || ui.pressed_element.unwrap() == element.id();

    if ui.mouse.pressed && hovered && can_be_pressed {
        ui.pressed_element = Some(id);
        ui.focused_element = Some(id);
        element.kind = UIElementKind::ButtonPressed;
    } else if hovered {
        element.kind = UIElementKind::ButtonHovered;
    }

    ui.hovering |= hovered;

    (hovered && just_released && can_be_pressed, element)
}
