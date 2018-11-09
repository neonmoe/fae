#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{UIState, Window, WindowSettings};

fn main() {
    let mut ui = UIState::new();
    let mut window = Window::create(WindowSettings {
        width: 160.0,
        height: 152.0,
        is_dialog: true,
        ..Default::default()
    })
    .unwrap();

    let mut counter: i64 = 0;
    while window.refresh(&mut ui, 0.8, 0.8, 0.8) {
        ui.label("counter", &format!("Counter: {}", counter));

        if ui.button("add", "Add") {
            counter += 1;
        }

        if ui.button("sub", "Subtract") {
            counter -= 1;
        }
    }
}
