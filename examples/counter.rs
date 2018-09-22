extern crate fungui as ui;
extern crate gl;

fn main() {
    let mut window = ui::Window::new(
        file!(),
        150.0,
        170.0,
        true,
        ui::resources::DEFAULT_UI_SPRITESHEET,
        ui::resources::DEFAULT_FONT,
    ).unwrap();

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
