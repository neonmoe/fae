//! Contains the functions that create the UI elements.

use layout::Rect;
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
    pub(crate) identifier: String,
    pub(crate) kind: UIElementKind,
    pub(crate) rect: Rect,
    pub(crate) alignment: Alignment,
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
        } = self.rect;
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

    let element = ui::new_element(identifier.to_owned(), UIElementKind::NoBackground);
    ui::draw_element(&element, display_text, true, None);
    state.insert_element(element);
}

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
        ui::draw_element(element, display_text, true, None);
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
        let Rect {
            left,
            top,
            right,
            bottom,
        } = element.rect;
        renderer::draw_quad((left, top, right, bottom), texcoords, color, z, tex_index);
    })
}

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

type CursorPosition = usize;

lazy_static! {
    static ref INPUT_STRINGS: Mutex<HashMap<u64, (String, CursorPosition)>> =
        Mutex::new(HashMap::new());
}

pub(crate) fn insert_input(focused_id: u64, input: char) {
    let mut strings = INPUT_STRINGS.lock().unwrap();
    if let Some(string) = strings.get_mut(&focused_id) {
        if !input.is_control() {
            string.0.push(input);
            string.1 += 1;
        } else if input == '\u{8}' && string.0.len() > 0 {
            string.0.pop();
            string.1 -= 1;
        }
    }
}

/// Creates an editable text field. Used for simple, label-like text
/// which is editable.
pub fn input(identifier: &str, default_text: &str) -> String {
    let mut state = UI_STATE.lock().unwrap();
    let mut strings = INPUT_STRINGS.lock().unwrap();

    let element = ui::new_element(identifier.to_owned(), UIElementKind::InputField);
    let id = element.id();

    let clicked = state.mouse.clicked();
    let focused = if clicked && element.is_point_inside(state.mouse.x, state.mouse.y) {
        state.focused_element = Some(id);
        true
    } else if let Some(focused_id) = state.focused_element {
        id == focused_id
    } else {
        false
    };

    if !strings.contains_key(&id) {
        strings.insert(id, (default_text.to_string(), default_text.len()));
    }
    let (text, cursor_location) = strings.get(&id).unwrap();
    let cursor = if focused && (Instant::now() - state.start_time).subsec_millis() > 500 {
        Some(*cursor_location)
    } else {
        None
    };
    ui::draw_element(&element, text, false, cursor);
    state.insert_element(element);

    text.clone()
}
