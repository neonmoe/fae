//! This is a very simple crate, mostly for profiling frame times.
#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{UIState, Window, WindowSettings};

fn main() {
    let mut ui = UIState::new();
    let mut window = Window::create(WindowSettings::default()).unwrap();

    while window.refresh(&mut ui, 0.8, 0.8, 0.8) {
        ui.label("frame-time", &format!("{:?}", window.avg_frame_duration()));
    }
}
