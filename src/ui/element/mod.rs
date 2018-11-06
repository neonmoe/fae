//! Contains the functions that create the UI elements.
mod input;
pub use self::input::*;

use rect::Rect;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use text::Alignment;
use ui::{self, OUTER_TILE_WIDTH, PADDING, UI_STATE};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum UIElementKind {
    NoBackground = -1,
    ButtonNormal = 0,
    ButtonHovered,
    ButtonPressed,
    InputField,
    KindCount,
}

#[derive(Clone, Debug)]
pub(crate) struct UIElement {
    pub identifier: String,
    pub kind: UIElementKind,
    pub rect: Rect,
    pub alignment: Alignment,
}

impl UIElement {
    pub(crate) fn id(&self) -> u64 {
        element_hash(&self.identifier)
    }

    pub(crate) fn is_point_inside(&self, x: f32, y: f32) -> bool {
        let (x0, y0, x1, y1) = self.rect.coords();
        !(x < x0 - PADDING - OUTER_TILE_WIDTH
            || x >= x1 + PADDING + OUTER_TILE_WIDTH
            || y < y0 - PADDING - OUTER_TILE_WIDTH
            || y >= y1 + PADDING + OUTER_TILE_WIDTH)
    }
}

pub(crate) fn element_hash(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Creates a text label. Used for displaying plain uneditable text.
pub fn label(identifier: &str, display_text: &str) {
    let mut state = UI_STATE.lock().unwrap();

    let element = ui::new_element(identifier.to_owned(), UIElementKind::NoBackground);
    ui::draw_element(&element, display_text, true, None);
    state.insert_element(element);
}

// TODO: Implement button ordering and only activate one button per press
fn button_meta<F: FnOnce(&UIElement)>(identifier: &str, render: F) -> bool {
    let mut state = UI_STATE.lock().unwrap();

    let mut element = ui::new_element(identifier.to_owned(), UIElementKind::ButtonNormal);
    let id = element.id();
    let hovered = element.is_point_inside(state.mouse.x, state.mouse.y);
    let just_released = state.mouse.clicked();
    let can_be_pressed =
        state.pressed_element.is_none() || state.pressed_element.unwrap() == element.id();

    if state.mouse.pressed && hovered && can_be_pressed {
        state.pressed_element = Some(id);
        state.focused_element = Some(id);
        element.kind = UIElementKind::ButtonPressed;
    } else if hovered {
        element.kind = UIElementKind::ButtonHovered;
    }

    state.hovering |= hovered;
    render(&element);
    state.insert_element(element);

    hovered && just_released && can_be_pressed
}

/// Renders a button that can be pressed, and returns whether or not
/// it was clicked.
pub fn button(identifier: &str, display_text: &str) -> bool {
    button_meta(identifier, |element| {
        ui::draw_element(element, display_text, false, None);
    })
}

/// Renders a button that can be pressed, and returns whether or not
/// it was clicked. In contrast to `button`, this version lets you
/// define the sprite used yourself. That said, the size and position
/// of the button is still controlled by the layout system.
pub fn button_image(
    identifier: &str,
    texcoords: (f32, f32, f32, f32),
    color: (u8, u8, u8, u8),
    z: f32,
    tex_index: usize,
) -> bool {
    use renderer;
    button_meta(identifier, |element| {
        renderer::draw_quad(element.rect.coords(), texcoords, color, z, tex_index);
    })
}
