//! This is a very simple crate, mostly for profiling frame times.
#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{Window, WindowSettings};

fn main() {
    let mut window = Window::create(WindowSettings::default()).unwrap();

    while window.refresh(0.8, 0.8, 0.8) {
        let ui = &mut window.ui;
        let frame_timer = &window.frame_timer;
        ui.label(
            "frame-time",
            &format!("{:?}", frame_timer.avg_frame_duration()),
        );
    }
}
