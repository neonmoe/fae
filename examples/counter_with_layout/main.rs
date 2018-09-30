//! This an identical program to the counter example, except for the
//! UI element layout which is more controlled in this one.

extern crate fungui;
extern crate gl;

use fungui::{element, layout, Window, WindowSettings};

fn main() {
    let mut window = Window::new(WindowSettings {
        width: 320.0,
        height: 48.0,
        is_dialog: true,
        ..Default::default()
    })
    .unwrap();

    let mut counter: i64 = 0;
    while window.refresh() {
        layout::define_layout(
            layout::Layout::new()
                .anchors(0.5, 0.5, 0.5, 0.5)
                .relative(-140.0, -8.0, -60.0, 8.0)
                .padding(20.0),
        );
        layout::define_direction(layout::Direction::Right);

        element::label("counter", &format!("Counter: {}", counter));

        if element::button("add", "Add") {
            counter += 1;
        }

        if element::button("sub", "Subtract") {
            counter -= 1;
        }
    }
}
