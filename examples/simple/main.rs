//! This is a very simple crate, mostly for profiling frame times.

extern crate fungui;
extern crate gl;

use fungui::{element, Window, WindowSettings};

fn main() {
    let mut window = Window::new(WindowSettings {
        ..Default::default()
    })
    .unwrap();

    while window.refresh() {
        element::label("frame-time", &format!("{:?}", window.avg_frame_duration()));
    }
}
