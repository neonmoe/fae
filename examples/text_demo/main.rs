//! For testing text input fields.
#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{element, Window, WindowSettings};

fn main() {
    let mut window = Window::new(WindowSettings {
        ..Default::default()
    })
    .unwrap();

    while window.refresh(0.8, 0.8, 0.8) {
        let _text = element::input("text", "default text!");
        // TODO: Implement a way of manipulating the input text (maybe by returning a &mut?) and then add a reset button
    }
}
