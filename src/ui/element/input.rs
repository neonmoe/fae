use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use ui::{self, UIElementKind, UI_STATE};

struct TextField {
    selection_time: Instant,
    pre_cursor_text: String,
    post_cursor_text: String,
    cursor_position: usize,
}

impl TextField {
    pub fn text(&self) -> String {
        format!("{}{}", self.pre_cursor_text, self.post_cursor_text)
    }
}

lazy_static! {
    static ref TEXT_FIELDS: Mutex<HashMap<u64, TextField>> = Mutex::new(HashMap::new());
}

fn insert_char(text_field: &mut TextField, input: char) {
    if !input.is_control() {
        let cursor = text_field.cursor_position;
        text_field.pre_cursor_text.push(input);
        text_field.cursor_position = (cursor + 1).min(text_field.pre_cursor_text.len());
    } else if input == '\u{8}' && !text_field.pre_cursor_text.is_empty() {
        text_field.pre_cursor_text.pop();
        text_field.cursor_position -= 1;
    }
}

pub(crate) fn insert_input(focused_id: u64, input: char) {
    let mut text_fields = TEXT_FIELDS.lock().unwrap();
    if let Some(text_field) = text_fields.get_mut(&focused_id) {
        insert_char(text_field, input);
    }
}

pub(crate) fn insert_input_str(focused_id: u64, input: &str) {
    let mut text_fields = TEXT_FIELDS.lock().unwrap();
    if let Some(text_field) = text_fields.get_mut(&focused_id) {
        for c in input.chars() {
            insert_char(text_field, c);
        }
    }
}

/// Creates an editable text field. Used for simple, label-like text
/// which is editable.
pub fn input(identifier: &str, default_text: &str) -> String {
    let mut state = UI_STATE.lock().unwrap();
    let mut text_fields = TEXT_FIELDS.lock().unwrap();

    let element = ui::new_element(identifier.to_owned(), UIElementKind::InputField);
    let id = element.id();
    if !text_fields.contains_key(&id) {
        text_fields.insert(
            id,
            TextField {
                selection_time: Instant::now(),
                pre_cursor_text: default_text.to_string(),
                post_cursor_text: String::new(),
                cursor_position: default_text.len(),
            },
        );
    }
    let field = text_fields.get_mut(&id).unwrap();

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

    let text = field.text();
    ui::draw_element(&element, &text, false, cursor);
    state.insert_element(element);

    text
}
