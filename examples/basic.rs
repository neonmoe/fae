#![windows_subsystem = "windows"]

use fae::text::{Alignment, TextRenderer};
use fae::{DrawCallParameters, Image, Renderer, Window, WindowSettings};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut window = Window::create(&WindowSettings::default()).unwrap();
    let mut renderer = Renderer::new(&window);
    let params = DrawCallParameters {
        image: {
            #[cfg(feature = "png")]
            let image = Image::from_png(&std::fs::read("examples/res/sprite.png")?)?;
            #[cfg(not(feature = "png"))]
            let image = Image::from_color(16, 16, &[0xFF, 0xFF, 0x00, 0xFF]);
            Some(image)
        },
        alpha_blending: false,
        ..Default::default()
    };
    let call = renderer.create_draw_call(params);

    let mut text = TextRenderer::create(&mut renderer);

    let mut should_quit = false;
    while window.refresh() && !should_quit {
        text.update_dpi_factor(window.dpi_factor); // TODO: Bad api

        if window
            .pressed_keys
            .contains(&glutin::VirtualKeyCode::Escape)
        {
            should_quit = true;
        }

        renderer.draw_quad(
            (0.0, 0.0, 640.0, 480.0), // The corner coordinates of the quad, in window coordinates (x0, y0, x1, y1)
            (0.0, 0.0, 1.0, 1.0), // The corner texture coordinates of the quad, in the 0..1 range (x0, y0, x1, y1)
            (1.0, 1.0, 1.0, 1.0), // The tint color of the texture (r, g, b, a)
            (0.0, 0.0, 0.0), // The rotation and pivot offset (radians, x, y). If x = 0.0 and y = 0.0, the quad will be rotated around its top-left coordinate, and this is shifted by x and y
            0.5, // The z coordinate of the quad, to specify which goes in front of what. Negative values are in front.
            &call, // The draw call during which to render this quad, practically this decides which texture is used
        );

        text.draw_text(
            "Some cool text!",    // The displayed text
            (10.0, 10.0, -0.6),   // The position (x, y, z)
            16.0,                 // The font size
            Alignment::Left,      // The text alignment (only applied if max_row_width is specified)
            (0.0, 0.0, 0.0, 1.0), // The text color
            None,                 // The maximum width of a row
            None,                 // The clipping area, if text overflows this, it gets cut off
        );

        text.compose_draw_call(&mut renderer);
        renderer.render(window.width, window.height);
        window.swap_buffers(Some(&renderer));
    }
    Ok(())
}
