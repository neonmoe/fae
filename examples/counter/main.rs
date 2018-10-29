#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{element, Window, WindowSettings};

fn main() {
    let mut window = Window::new(WindowSettings {
        width: 160.0,
        height: 152.0,
        is_dialog: true,
        ..Default::default()
    })
    .unwrap();

    let mut counter: i64 = 0;
    while window.refresh(0.8, 0.8, 0.8) {
        element::label("counter", &format!("Counter: {}", counter));

        if element::button("add", "Add") {
            counter += 1;
        }

        if element::button("sub", "Subtract") {
            counter -= 1;
        }
    }
}
