extern crate fungui;
extern crate gl;

fn main() {
    let mut window = fungui::Window::new("fungui/examples/basic.rs", 640.0, 480.0);
    fungui::initialize();
    let mut time = 0f32;
    while window.refresh() {
        time += 0.016;
        fungui::draw_quad(
            170.0 + 50.0 * (time * 2.0).sin(),
            90.0 + 50.0 * (time * 2.0).cos(),
            300.0,
            300.0,
            0.0,
            0.0,
            0.0,
            1.0,
            1.0,
        );
        fungui::render(window.width, window.height);
    }
}
