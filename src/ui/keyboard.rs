//! Get your keypresses here.
use super::{ModifiersState, VirtualKeyCode, UI_STATE};

/// Returns true if the given key is held, and the modifiers were
/// active when the key was initially pressed. If you don't care
/// about the modifiers, leave `modifiers` as None.
pub fn key_held(keycode: VirtualKeyCode, modifiers: Option<ModifiersState>) -> bool {
    let state = UI_STATE.lock().unwrap();
    if !state.keys.contains_key(&keycode) {
        false
    } else {
        let key_state = state.keys[&keycode];
        key_state.pressed && (modifiers.is_none() || key_state.modifiers == modifiers.unwrap())
    }
}

/// Returns true if the given key was just released, and the
/// modifiers were active when the key was initially pressed. If
/// you don't care about the modifiers, leave `modifiers` as None.
pub fn key_typed(keycode: VirtualKeyCode, modifiers: Option<ModifiersState>) -> bool {
    let state = UI_STATE.lock().unwrap();
    if !state.keys.contains_key(&keycode) {
        false
    } else {
        let key_state = state.keys[&keycode];
        !key_state.pressed
            && key_state.last_pressed
            && (modifiers.is_none() || key_state.modifiers == modifiers.unwrap())
    }
}
