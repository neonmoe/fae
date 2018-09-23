use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum UIElementKind {
    NoBackground = -1,
    ButtonNormal = 0,
    ButtonHovered = 1,
    ButtonPressed = 2,
    Panel = 3,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
}

#[derive(Clone, Copy)]
pub struct UIElementDimensions {
    pub relative: Rect,
    pub anchors: Rect,
}

impl UIElementDimensions {
    pub(crate) fn absolute(&self) -> (f32, f32, f32, f32) {
        let lock = WINDOW_DIMENSIONS.lock().unwrap();
        let (width, height) = *lock;
        let x0o = width * self.anchors.x0;
        let x1o = width * self.anchors.x1;
        let y0o = height * self.anchors.y0;
        let y1o = height * self.anchors.y1;
        let Rect { x0, y0, x1, y1 } = self.relative;
        (x0 + x0o, y0 + y0o, x1 + x1o, y1 + y1o)
    }
}

#[derive(Clone)]
pub(crate) struct UIElement {
    pub(crate) identifier: String,
    pub(crate) kind: UIElementKind,
    pub(crate) dimensions: UIElementDimensions,
}

impl UIElement {
    pub(crate) fn id(&self) -> u64 {
        element_hash(&self.identifier)
    }

    pub(crate) fn is_point_inside(&self, x: f32, y: f32) -> bool {
        let (x0, y0, x1, y1) = self.dimensions.absolute();
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

pub fn label(label: &str, display_text: &str) {
    let mut state = UI_STATE.lock().unwrap();

    let element = new_element(&state, label.to_owned(), UIElementKind::Panel);
    draw_element(&element, display_text);
    state.insert_element(element);
}

pub fn button(label: &str, display_text: &str) -> bool {
    let mut state = UI_STATE.lock().unwrap();

    let mut element = new_element(&state, label.to_owned(), UIElementKind::ButtonNormal);
    let hovered = element.is_point_inside(state.mouse.x, state.mouse.y);
    let just_released = !state.mouse.pressed && state.mouse.last_pressed;
    let can_be_pressed =
        state.pressed_element.is_none() || state.pressed_element.unwrap() == element.id();

    if state.mouse.pressed && hovered && can_be_pressed {
        element.kind = UIElementKind::ButtonPressed;
        state.pressed_element = Some(element.id());
    } else if hovered {
        element.kind = UIElementKind::ButtonHovered;
    }

    state.hovering |= hovered;
    draw_element(&element, display_text);
    state.insert_element(element);

    hovered && just_released && can_be_pressed
}
