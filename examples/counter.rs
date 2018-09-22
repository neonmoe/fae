extern crate fungui as ui;
extern crate gl;

fn main() {
    let mut window = ui::Window::new(ui::WindowSettings {
        width: 150.0,
        height: 170.0,
        is_dialog: true,
        ..Default::default()
    }).unwrap();

    let mut counter: i64 = 0;
    while window.refresh() {
        ui::label(&format!("Counter: {}", counter));

        if ui::button("Add") {
            counter += 1;
        }

        if ui::button("Subtract") {
            counter -= 1;
        }
    }
}
