extern crate fungui as ui;
extern crate gl;

fn main() {
    let mut window = ui::Window::new(file!(), 320.0, 240.0);
    ui::initialize();
    let mut counter: i64 = 0;
    while window.refresh() {
        ui::label(&format!("Counter: {}", counter));
        if ui::button("Add") {
            counter += 1;
            println!("Counter: {}", counter);
        }
        if ui::button("Subtract") {
            counter -= 1;
            println!("Counter: {}", counter);
        }
    }
}
