use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum UIElementType {
    NoBackground = -1,
    ButtonNormal = 0,
    ButtonHovered = 1,
    ButtonPressed = 2,
    Panel = 3,
}

#[derive(Clone)]
pub(crate) struct UIElement {
    pub(crate) identifier: String,
    pub(crate) t: UIElementType,
    pub(crate) x0: f32,
    pub(crate) y0: f32,
    pub(crate) x1: f32,
    pub(crate) y1: f32,
    pub(crate) anchor_x: f32,
    pub(crate) anchor_y: f32,
}

impl UIElement {
    pub(crate) fn id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.identifier.hash(&mut hasher);
        hasher.finish()
    }

    pub(crate) fn is_point_inside(&self, x: f32, y: f32) -> bool {
        !(x < self.x0 - PADDING - OUTER_TILE_WIDTH
            || x >= self.x1 + PADDING + OUTER_TILE_WIDTH
            || y < self.y0 - PADDING - OUTER_TILE_WIDTH
            || y >= self.y1 + PADDING + OUTER_TILE_WIDTH)
    }
}

pub fn label(label: &str) {
    let mut state = UI_STATE.lock().unwrap();

    let element = new_element(&state, label.to_owned(), UIElementType::Panel);
    draw_element(&element, label);
    state.insert_element(element);
}

pub fn button(label: &str) -> bool {
    let mut state = UI_STATE.lock().unwrap();

    let mut element = new_element(&state, label.to_owned(), UIElementType::ButtonNormal);
    let hovered = element.is_point_inside(state.mouse.x, state.mouse.y);
    let just_released = !state.mouse.pressed && state.mouse.last_pressed;
    let can_be_pressed =
        state.pressed_element.is_none() || state.pressed_element.unwrap() == element.id();

    if state.mouse.pressed && hovered && can_be_pressed {
        element.t = UIElementType::ButtonPressed;
        state.pressed_element = Some(element.id());
    } else if hovered {
        element.t = UIElementType::ButtonHovered;
    }

    state.hovering |= hovered;
    draw_element(&element, label);
    state.insert_element(element);

    hovered && just_released && can_be_pressed
}
