//! This an identical program to the counter example, except for the
//! UI element layout.
//!
//! ### Note
//!
//! This is a bad way to control the layout of your elements. Loading
//! an external layout configuration file is much more flexible. There
//! will be a way to configure your UI during runtime, with the option
//! of exporting the UI's state into said configuration file.

extern crate fungui;
extern crate gl;

use fungui::{element, layout, Window, WindowSettings};

fn main() {
    let mut window = Window::new(WindowSettings {
        width: 300.0,
        height: 48.0,
        is_dialog: true,
        ..Default::default()
    }).unwrap();

    layout::define_element_layout(
        "counter",
        layout::UIElementLayout::new()
            .relative(16.0, 16.0, -16.0, -16.0)
            .anchors(0.0, 0.0, 0.4, 1.0)
            .alignment(layout::Alignment::Left),
    );

    layout::define_element_layout(
        "add",
        layout::UIElementLayout::new()
            .relative(16.0, 16.0, -8.0, -16.0)
            .anchors(0.4, 0.0, 0.7, 1.0)
            .alignment(layout::Alignment::Center),
    );

    layout::define_element_layout(
        "sub",
        layout::UIElementLayout::new()
            .relative(8.0, 16.0, -16.0, -16.0)
            .anchors(0.7, 0.0, 1.0, 1.0)
            .alignment(layout::Alignment::Center),
    );

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
