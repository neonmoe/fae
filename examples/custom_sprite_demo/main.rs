//! Here's a minimalistic example of printing custom sprites with the
//! crate.
#![windows_subsystem = "windows"]

extern crate fungui;

use fungui::{renderer, Window, WindowSettings};

fn main() {
    let mut window = Window::create(WindowSettings::default()).unwrap();

    let spritesheet_tex_index = renderer::create_draw_call(include_bytes!("test.png"));

    while window.refresh(0.8, 0.8, 0.8) {
        renderer::draw_quad(
            (70.0, 70.0, 170.0, 170.0), // Left, top, right and bottom coordinates
            (0.0, 0.0, 1.0, 1.0),       // Same as before but for the texture
            (0xFF, 0xFF, 0xFF, 0xFF),   // Tint of the sprite (all 0xFF is white, so no effect)
            0.0,                        // Z-coordinate of the sprite, negative is in front
            spritesheet_tex_index,      // The spritesheet texture index
        );

        renderer::draw_quad(
            (135.0, 135.0, 185.0, 185.0), // Left, top, right and bottom coordinates
            (0.5, 0.5, 1.0, 1.0),         // Same as before but for the texture
            (0x22, 0x88, 0xFF, 0xFF),     // Tint of the sprite (all 0xFF is white, so no effect)
            -0.1,                         // Z-coordinate of the sprite, negative is in front
            spritesheet_tex_index,        // The spritesheet texture index
        );

        renderer::draw_quad(
            (150.0, 100.0, 200.0, 150.0), // Left, top, right and bottom coordinates
            (0.5, 0.25, 1.0, 0.75),       // Same as before but for the texture
            (0xFF, 0x88, 0x22, 0xFF),     // Tint of the sprite (all 0xFF is white, so no effect)
            0.1,                          // Z-coordinate of the sprite, negative is in front
            spritesheet_tex_index,        // The spritesheet texture index
        );
    }
}
