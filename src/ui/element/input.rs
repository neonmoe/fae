use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use text::TextCursor;
use ui::{self, UIElementKind, UI_STATE};

struct TextField {
    selection_time: Instant,
    pre_cursor_text: String,
    post_cursor_text: String,
    cursor: TextCursor,
}

impl TextField {
    pub fn text(&self) -> String {
        format!("{}{}", self.pre_cursor_text, self.post_cursor_text)
    }
}

lazy_static! {
    static ref TEXT_FIELDS: Mutex<HashMap<u64, TextField>> = Mutex::new(HashMap::new());
}

pub(crate) fn move_cursor(modifier: i32) {
    if modifier == 0 {
        return;
    }
    let state = UI_STATE.lock().unwrap();
    if let Some(id) = state.focused_element {
        let mut text_fields = TEXT_FIELDS.lock().unwrap();
        if let Some(field) = text_fields.get_mut(&id) {
            if modifier > 0 {
                for _ in 0..modifier {
                    let new_text = field.post_cursor_text.clone();
                    let mut chars = new_text.chars();
                    if let Some(c) = chars.next() {
                        field.pre_cursor_text += &c.to_string();
                        field.cursor.index += 1;
                    } else {
                        break;
                    }
                    field.post_cursor_text = chars.collect();
                }
            } else {
                for _ in 0..(-modifier) {
                    let new_text = field.pre_cursor_text.clone();
                    let mut chars = new_text.chars();
                    if let Some(c) = chars.next_back() {
                        field.post_cursor_text = c.to_string() + &field.post_cursor_text;
                        field.cursor.index -= 1;
                    } else {
                        break;
                    }
                    field.pre_cursor_text = chars.collect();
                }
            }
            field.selection_time = Instant::now();
        }
    }
}

fn insert_char(text_field: &mut TextField, input: char) {
    if !input.is_control() {
        text_field.pre_cursor_text.push(input);
        let cursor = text_field.cursor.index + 1;
        text_field.cursor.index = cursor.min(text_field.pre_cursor_text.len());
    } else if input == '\u{8}' && !text_field.pre_cursor_text.is_empty() {
        text_field.pre_cursor_text.pop();
        text_field.cursor.index -= 1;
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
                cursor: TextCursor::new(default_text.len(), false),
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

    field.cursor.blink_visibility =
        focused && (Instant::now() - field.selection_time).subsec_millis() < 500;

    let text = field.text();
    ui::draw_element(&element, &text, false, Some(&mut field.cursor));
    state.insert_element(element);

    text
}
