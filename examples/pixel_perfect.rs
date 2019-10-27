//! This example is used to debug that fae renders textures pixel-perfectly if the texture's resolution matches the quad's coordinates.
#![windows_subsystem = "windows"]

use fae::{
    text::{Alignment, TextRenderer},
    DrawCallParameters, Image, Renderer, Window, WindowSettings,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut window = Window::create(&WindowSettings {
        multisample: 0,
        ..Default::default()
    })?;
    let mut renderer = Renderer::new(&window);
    let mut text = TextRenderer::create_simple(&mut renderer, false);
    let call = renderer.create_draw_call(DrawCallParameters {
        image: Some(Image::from_png(include_bytes!("res/sprite_8x8.png"))?),
        magnification_smoothing: false,
        ..Default::default()
    });

    while window.refresh() {
        renderer.set_dpi_factor(window.dpi_factor);

        text.draw_text(
            "Some text for seeing if everything breaks suddenly or something.\nLet's hope that doesn't happen.",
            (300.0, 10.0, 0.0),
            32.0 / window.dpi_factor,
            Alignment::Center,
            (0.0, 0.0, 0.0, 1.0),
            Some(320.0),
            None,
        );

        text.debug_draw_glyph_cache(&mut renderer, (20.0, 20.0, 148.0, 148.0), -1.0);
        renderer
            .draw(&call, -0.9)
            .with_coordinates(20.0, 20.0, 128.0, 128.0)
            .with_color(0.9, 0.9, 0.9, 1.0)
            .finish();

        let mut y = 158.0;
        let x = 32.0;
        for i in 0..4 {
            let size = 8.0 * 2.0f32.powf(i as f32) / window.dpi_factor;
            let px = 1.0 / window.dpi_factor;
            let offset = -0.5 / size * px;
            renderer
                .draw(&call, -0.9)
                .with_coordinates(x, y, size, size)
                .with_uvs(0.0 + offset, 0.0 + offset, 1.0 + offset, 1.0 + offset)
                .finish();
            y += 10.0 + size;
        }

        let mut y = 158.0;
        let x = 128.0;
        for i in 0..4 {
            let size = 8.0 * 2.0f32.powf(i as f32) / window.dpi_factor;
            renderer
                .draw(&call, -0.9)
                .with_coordinates(x, y, size, size)
                .with_texture_coordinates(0, 0, 8, 8)
                .finish();
            y += 10.0 + size;
        }

        text.compose_draw_call(&mut renderer);
        renderer.render(window.width, window.height);
        window.swap_buffers(Some(&renderer));
    }
    Ok(())
}
