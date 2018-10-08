//! This an identical program to the `counter` example, except that
//! the UI element layout is more controlled in this one.
#![windows_subsystem = "windows"]

extern crate fungui;
extern crate gl;

use fungui::{element, layout, Window, WindowSettings};

fn main() {
    let mut window = Window::new(WindowSettings {
        width: 320.0,
        height: 92.0,
        is_dialog: true,
        ..Default::default()
    })
    .unwrap();

    let mut counter: i64 = 0;
    while window.refresh(0.8, 0.8, 0.8) {
        layout::push_padding(20.0);
        layout::push_direction(layout::Direction::Right);

        layout::push_rect(
            layout::screen_x(0.5) - 140.0,
            layout::screen_y(0.5) - 26.0,
            280.0,
            16.0,
        );
        element::label("counter", &format!("Counter: {}", counter));
        layout::pop_rect();

        layout::push_rect(
            layout::screen_x(0.5) - 140.0,
            layout::screen_y(0.5) + 10.0,
            130.0,
            16.0,
        );
        if element::button("add", "Add") {
            counter += 1;
        }

        if element::button("sub", "Subtract") {
            counter -= 1;
        }
        layout::pop_rect();

        layout::pop_direction();
        layout::pop_padding();
    }
}
