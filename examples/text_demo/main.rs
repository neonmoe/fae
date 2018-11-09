//! For testing text input fields.
#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{UIState, Window, WindowSettings};

fn main() {
    let mut ui = UIState::new();
    let mut window = Window::create(WindowSettings::default()).unwrap();

    while window.refresh(&mut ui, 0.8, 0.8, 0.8) {
        let _text = ui.input("text", "default text!");
        // TODO: Implement a way of manipulating the input text (maybe
        // by returning a &mut?) and then add a reset button
    }
}
