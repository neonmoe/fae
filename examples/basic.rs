use fae::text::{Alignment, TextRenderer};
use fae::{DrawCallParameters, Image, Renderer, Window, WindowSettings};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::create(&WindowSettings::default()).unwrap();
    let mut renderer = Renderer::new(&window);
    let params = DrawCallParameters {
        image: {
            #[cfg(feature = "png")]
            let image = Image::from_png(include_bytes!("res/sprite.png"))?;
            #[cfg(not(feature = "png"))]
            let image = Image::from_color(16, 16, &[0xFF, 0xFF, 0x00, 0xFF]);
            Some(image)
        },
        alpha_blending: false,
        ..Default::default()
    };
    let call = renderer.create_draw_call(params);

    let mut text = TextRenderer::with_font8x8(&mut renderer, true);

    let mut should_quit = false;
    while window.refresh() && !should_quit {
        renderer.set_dpi_factor(window.dpi_factor);
        text.set_dpi_factor(window.dpi_factor);

        if window
            .pressed_keys
            .contains(&glutin::VirtualKeyCode::Escape)
        {
            should_quit = true;
        }

        renderer
            .draw(&call, 0.5)
            .with_coordinates((0.0, 0.0, 640.0, 480.0))
            .with_texture_coordinates((0, 0, 1240, 920))
            .finish();

        text.draw("Some cool text!", 10.0, 10.0, 0.6, 16.0)
            .with_alignment(Alignment::Left)
            .with_color((0.0, 0.5, 0.1, 1.0))
            .finish();

        text.compose_draw_call(&mut renderer);
        renderer.render(window.width, window.height);
        window.swap_buffers(Some(&renderer));
    }
    Ok(())
}
