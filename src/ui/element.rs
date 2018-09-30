//! Contains the functions that create the UI elements.

use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum UIElementKind {
    NoBackground = -1,
    ButtonNormal = 0,
    ButtonHovered = 1,
    ButtonPressed = 2,
    Panel = 3,
}

#[derive(Clone, Debug)]
pub(crate) struct UIElement {
    pub(crate) identifier: String,
    pub(crate) kind: UIElementKind,
    pub(crate) layout: Layout,
}

impl UIElement {
    pub(crate) fn id(&self) -> u64 {
        element_hash(&self.identifier)
    }

    pub(crate) fn is_point_inside(&self, x: f32, y: f32) -> bool {
        let Rect {
            left,
            top,
            right,
            bottom,
        } = self.layout.absolute();
        !(x < left - PADDING - OUTER_TILE_WIDTH
            || x >= right + PADDING + OUTER_TILE_WIDTH
            || y < top - PADDING - OUTER_TILE_WIDTH
            || y >= bottom + PADDING + OUTER_TILE_WIDTH)
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

    let element = new_element(identifier.to_owned(), UIElementKind::Panel);
    draw_element(&element, display_text);
    state.insert_element(element);
}

/// Renders a button that can be pressed, and returns whether or not
/// it was clicked.
pub fn button(identifier: &str, display_text: &str) -> bool {
    let mut state = UI_STATE.lock().unwrap();

    let mut element = new_element(identifier.to_owned(), UIElementKind::ButtonNormal);
    let hovered = element.is_point_inside(state.mouse.x, state.mouse.y);
    let just_released = !state.mouse.pressed && state.mouse.last_pressed;
    let can_be_pressed =
        state.pressed_element.is_none() || state.pressed_element.unwrap() == element.id();

    if state.mouse.pressed && hovered && can_be_pressed {
        state.pressed_element = Some(element.id());
        element.kind = UIElementKind::ButtonPressed;
    } else if hovered {
        element.kind = UIElementKind::ButtonHovered;
    }

    state.hovering |= hovered;
    draw_element(&element, display_text);
    state.insert_element(element);

    hovered && just_released && can_be_pressed
}
