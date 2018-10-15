use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use ui::{self, UIElementKind, UI_STATE};

struct TextField {
    selection_time: Instant,
    text: String,
    cursor_position: usize,
}

lazy_static! {
    static ref INPUT_STRINGS: Mutex<HashMap<u64, TextField>> = Mutex::new(HashMap::new());
}

pub(crate) fn insert_input(focused_id: u64, input: char) {
    let mut strings = INPUT_STRINGS.lock().unwrap();
    if let Some(string) = strings.get_mut(&focused_id) {
        if !input.is_control() {
            string.text.push(input);
            string.cursor_position += 1;
        } else if input == '\u{8}' && string.text.len() > 0 {
            string.text.pop();
            string.cursor_position -= 1;
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
    if !strings.contains_key(&id) {
        strings.insert(
            id,
            TextField {
                selection_time: Instant::now(),
                text: default_text.to_string(),
                cursor_position: default_text.len(),
            },
        );
    }
    let field = strings.get_mut(&id).unwrap();

    let clicked = state.mouse.clicked();
    let focused = if clicked && element.is_point_inside(state.mouse.x, state.mouse.y) {
        field.selection_time = Instant::now();
        state.focused_element = Some(id);
        true
    } else if let Some(focused_id) = state.focused_element {
        id == focused_id
    } else {
        false
    };

    let cursor = if focused && (Instant::now() - field.selection_time).subsec_millis() < 500 {
        Some(field.cursor_position)
    } else {
        None
    };

    ui::draw_element(&element, &field.text, false, cursor);
    state.insert_element(element);

    field.text.clone()
}
