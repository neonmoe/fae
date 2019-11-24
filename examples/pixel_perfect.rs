//! This example is used to debug that fae renders textures pixel-perfectly if the texture's resolution matches the quad's coordinates.
mod common;

use common::WindowSettings;
use fae::{glutin::dpi::LogicalSize, DrawCallParameters, Image, Window};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::create(
        WindowSettings {
            multisample: 8,
            width: 500.0,
            height: 200.0,
            ..Default::default()
        }
        .into(),
    )?;
    let (mut renderer, mut text) = common::create_renderers(&window);
    let call = renderer.create_draw_call(DrawCallParameters {
        image: Some(Image::from_png(include_bytes!("res/sprite_8x8.png"))?),
        minification_smoothing: true,
        magnification_smoothing: true,
        ..Default::default()
    });

    window.refresh();
    let (w, h) = (
        window.width / window.dpi_factor,
        window.height / window.dpi_factor,
    );
    window
        .get_window()
        .set_inner_size(LogicalSize::new(w.into(), h.into()));

    while window.refresh() {
        renderer.set_dpi_factor(window.dpi_factor);
        text.prepare_new_frame(
            &mut renderer,
            window.dpi_factor,
            window.width,
            window.height,
        );
        let font_size = 8.0 / window.dpi_factor;

        let mut y = 0.0;
        let x = 0.0;
        for i in 0..4 {
            let size = 8.0 * 2.0f32.powf(i as f32) / window.dpi_factor;
            renderer
                .draw(&call, -0.9)
                .with_coordinates((x, y, size, size))
                .with_texture_coordinates((0, 0, 8, 8))
                .finish();

            let s = format!(
                "<- x{} zoom{}",
                i + 1,
                if i == 0 {
                    ", should be 1:1 with examples/res/sprite_8x8.png"
                } else {
                    ""
                }
            );
            text.draw(
                s,
                x + size + 10.0,
                y + (size - font_size) / 2.0,
                0.0,
                font_size,
            )
            .with_cacheable(true)
            .finish();

            y += 10.0 + size;
        }

        let mut y = 20.0;
        let x = 150.0;
        for i in 0..4 {
            let size = 8.0 * 2.0f32.powf(i as f32) / window.dpi_factor;
            renderer
                .draw(&call, -0.9)
                .with_coordinates((x, y, size, size))
                .with_texture_coordinates((0, 0, 8, 8))
                .finish();
            y += 10.0 + size;
        }

        text.draw("with pixel align =", 175.0, 30.0, 0.0, font_size)
            .with_cacheable(true)
            .finish();

        let mut y = 20.0;
        let x = 314.0;
        for i in 0..4 {
            let size = 8.0 * 2.0f32.powf(i as f32) / window.dpi_factor;
            renderer
                .draw(&call, -0.9)
                .with_coordinates((x, y, size, size))
                .with_texture_coordinates((0, 0, 8, 8))
                .with_pixel_alignment()
                .finish();
            y += 10.0 + size;
        }

        text.compose_draw_call(&mut renderer);
        renderer.render(window.width, window.height);
        window.swap_buffers(Some(&renderer));
    }
    Ok(())
}
