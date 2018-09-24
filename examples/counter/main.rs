extern crate fungui;
extern crate gl;

use fungui::{element, Window, WindowSettings};

fn main() {
    let mut window = Window::new(WindowSettings {
        width: 150.0,
        height: 170.0,
        is_dialog: true,
        ..Default::default()
    }).unwrap();

    let mut counter: i64 = 0;
    while window.refresh() {
        element::label("counter", &format!("Counter: {}", counter));

        if element::button("add", "Add") {
            counter += 1;
        }

        if element::button("sub", "Subtract") {
            counter -= 1;
        }
    }
}
