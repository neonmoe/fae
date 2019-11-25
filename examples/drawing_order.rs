/// This example demonstrates the erroneus drawing that happens when
/// drawing alpha-blended sprites on top of each other in the wrong
/// order. For multiple sprites to properly blend, the ones in the
/// back have to be drawn before the ones in the front.
mod common;

use common::WindowSettings;
use fae::text::Alignment;
use fae::{DrawCallParameters, Image, Window};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::create(WindowSettings::default().into()).unwrap();
    let (mut renderer, mut text) = common::create_renderers(&window);
    let params = DrawCallParameters {
        image: Some(Image::from_png(include_bytes!(
            "res/transparent_sprite.png",
        ))?),
        ..Default::default()
    };
    let call_below = renderer.create_draw_call(params.clone());
    let call_above = renderer.create_draw_call(params.clone());

    let mut should_quit = false;
    while window.refresh() && !should_quit {
        renderer.set_dpi_factor(window.dpi_factor);
        text.prepare_new_frame(
            &mut renderer,
            window.dpi_factor,
            window.width,
            window.height,
        );

        if window
            .pressed_keys
            .contains(&glutin::VirtualKeyCode::Escape)
        {
            should_quit = true;
        }

        {
            let s = "Correct: The draw call of the sprite in front is drawn last:";
            text.draw(s, 10.0, 10.0, 0.6, 10.0)
                .with_max_width(280.0)
                .finish();

            renderer
                .draw(&call_below, -0.5)
                .with_coordinates((40.0, 50.0, 128.0, 128.0))
                .with_texture_coordinates((0, 0, 8, 8))
                .finish();
            renderer
                .draw(&call_above, 0.5)
                .with_coordinates((40.0 + 48.0, 50.0 + 48.0, 128.0, 128.0))
                .with_texture_coordinates((0, 0, 8, 8))
                .finish();
        }

        {
            let s = "Not correct: The draw call of the sprite in front is drawn first:";
            text.draw(s, 330.0, 10.0, 0.6, 10.0)
                .with_max_width(280.0)
                .finish();

            renderer
                .draw(&call_above, -0.4)
                .with_coordinates((360.0, 50.0, 128.0, 128.0))
                .with_texture_coordinates((0, 0, 8, 8))
                .finish();
            renderer
                .draw(&call_below, 0.4)
                .with_coordinates((360.0 + 48.0, 50.0 + 48.0, 128.0, 128.0))
                .with_texture_coordinates((0, 0, 8, 8))
                .finish();
        }

        let s = "The draw call drawing order is decided by the highest Z-coordinate of each call, ascending.\n\nIn this example, call_below's highest Z coordinate is 0.4, and call_above's is 0.5. Therefore, call_below is drawn first.\n(In the right example, call_below is the one in front.)";
        text.draw(s, 80.0, 250.0, 0.6, 10.0)
            .with_max_width(400.0)
            .with_alignment(Alignment::Center)
            .finish();

        text.compose_draw_call(&mut renderer);
        renderer.render(window.width, window.height);
        window.swap_buffers(Some(&renderer));
    }
    Ok(())
}
