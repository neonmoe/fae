#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{Window, WindowSettings};

fn main() {
    let mut window = Window::create(WindowSettings::default()).unwrap();

    let mut counter: i64 = 0;
    while window.refresh(0.8, 0.8, 0.8) {
        let ui = &mut window.ui;
        ui.label("counter", &format!("Counter: {}", counter));

        if ui.button("add", "Add") {
            counter += 1;
        }

        if ui.button("sub", "Subtract") {
            counter -= 1;
        }
    }
}
