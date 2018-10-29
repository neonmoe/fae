//! This is a very simple crate, mostly for profiling frame times.
#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{element, Window, WindowSettings};

fn main() {
    let mut window = Window::new(WindowSettings {
        ..Default::default()
    })
    .unwrap();

    while window.refresh(0.8, 0.8, 0.8) {
        element::label("frame-time", &format!("{:?}", window.avg_frame_duration()));
    }
}
