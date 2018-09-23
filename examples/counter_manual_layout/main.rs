//! This an identical program to the counter example, except for the
//! UI element layout.
//!
//! ### Note
//!
//! This is a bad way to control the layout of your elements. Loading
//! an external layout configuration file is much more flexible. There
//! will be a way to configure your UI during runtime, with the option
//! of exporting the UI's state into said configuration file.

extern crate fungui as ui;
extern crate gl;

fn main() {
    let mut window = ui::Window::new(ui::WindowSettings {
        width: 300.0,
        height: 48.0,
        is_dialog: true,
        ..Default::default()
    }).unwrap();

    ui::define_element_dimensions(
        "counter",
        ui::UIElementDimensions {
            relative: ui::Rect {
                x0: 16.0,
                y0: 16.0,
                x1: -16.0,
                y1: -16.0,
            },
            anchors: ui::Rect {
                x0: 0.0,
                y0: 0.0,
                x1: 0.4,
                y1: 1.0,
            },
        },
    );

    ui::define_element_dimensions(
        "add",
        ui::UIElementDimensions {
            relative: ui::Rect {
                x0: 16.0,
                y0: 16.0,
                x1: -8.0,
                y1: -16.0,
            },
            anchors: ui::Rect {
                x0: 0.4,
                y0: 0.0,
                x1: 0.7,
                y1: 1.0,
            },
        },
    );

    ui::define_element_dimensions(
        "sub",
        ui::UIElementDimensions {
            relative: ui::Rect {
                x0: 8.0,
                y0: 16.0,
                x1: -16.0,
                y1: -16.0,
            },
            anchors: ui::Rect {
                x0: 0.7,
                y0: 0.0,
                x1: 1.0,
                y1: 1.0,
            },
        },
    );

    let mut counter: i64 = 0;
    while window.refresh() {
        ui::label("counter", &format!("Counter: {}", counter));

        if ui::button("add", "Add") {
            counter += 1;
        }

        if ui::button("sub", "Subtract") {
            counter -= 1;
        }
    }
}
