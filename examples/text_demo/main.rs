//! For testing text input fields.
#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{Window, WindowSettings};

fn main() {
    let mut window = Window::create(WindowSettings::default()).unwrap();

    while window.refresh(0.8, 0.8, 0.8) {
        let ui = &mut window.ui;
        let _text = ui.input("text", "default text!");
        // TODO: Implement a way of manipulating the input text (maybe
        // by returning a &mut?) and then add a reset button
    }
}
