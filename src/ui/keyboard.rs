//! Get your keypresses here.
use super::{ModifiersState, VirtualKeyCode};
use std::collections::HashMap;

/// Represents the status of a key on the keyboard.
#[derive(Clone, Copy, Debug)]
pub struct KeyStatus {
    /// The key this status describes.
    pub keycode: VirtualKeyCode,
    /// The modifiers which were pressed with the key.
    pub modifiers: ModifiersState,
    /// Was the key pressed during the previous frame?
    pub last_pressed: bool,
    /// Is the key being pressed currently?
    pub pressed: bool,
}

/// Contains the state of the keyboard, ie. which keys are pressed.
pub struct Keyboard {
    keys: HashMap<VirtualKeyCode, KeyStatus>,
}

impl Keyboard {
    pub(crate) fn new() -> Keyboard {
        Keyboard {
            keys: HashMap::new(),
        }
    }

    pub(crate) fn update(&mut self, key_inputs: Vec<KeyStatus>) {
        for key in self.keys.iter_mut().map(|(_, key_status)| key_status) {
            key.last_pressed = key.pressed;
        }
        for mut key_input in key_inputs {
            let keycode = key_input.keycode;
            key_input.last_pressed = {
                if !self.keys.contains_key(&keycode) {
                    false
                } else {
                    self.keys[&keycode].pressed
                }
            };
            self.keys.insert(keycode, key_input);
        }
    }

    /// Returns true if the given key is held, and the modifiers were
    /// active when the key was initially pressed. If you don't care
    /// about the modifiers, leave `modifiers` as None.
    pub fn key_held(&mut self, keycode: VirtualKeyCode, modifiers: Option<ModifiersState>) -> bool {
        if !self.keys.contains_key(&keycode) {
            false
        } else {
            let key_state = self.keys[&keycode];
            key_state.pressed && (modifiers.is_none() || key_state.modifiers == modifiers.unwrap())
        }
    }

    /// Returns true if the given key was just released, and the
    /// modifiers were active when the key was initially pressed. If
    /// you don't care about the modifiers, leave `modifiers` as None.
    pub fn key_typed(
        &mut self,
        keycode: VirtualKeyCode,
        modifiers: Option<ModifiersState>,
    ) -> bool {
        if !self.keys.contains_key(&keycode) {
            false
        } else {
            let key_state = self.keys[&keycode];
            !key_state.pressed
                && key_state.last_pressed
                && (modifiers.is_none() || key_state.modifiers == modifiers.unwrap())
        }
    }
}
