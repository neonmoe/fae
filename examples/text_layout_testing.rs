//! This example writes text in varying ways to test that the layout
//! functionality works correctly.
#![windows_subsystem = "windows"]

use fae::{
    text::{Alignment, TextRenderer},
    DrawCallParameters, Mouse, Renderer, Window, WindowSettings,
};
use std::error::Error;

static LOREM_IPSUM: &'static str = "Perferendis officiis ut provident sit eveniet ipsa eos. Facilis delectus at laudantium nemo. Sed ipsa natus perferendis dignissimos odio deserunt omnis.

Reprehenderit voluptas provident eveniet eos hic et maiores. Voluptatum totam sit quisquam consequatur atque sunt animi. Rem deleniti ex quia consequatur voluptate nostrum.

In earum architecto qui sunt provident. Vitae rerum molestiae dolorem praesentium fugit nostrum voluptas omnis. Rem sint voluptatem autem eum. Tempore velit maxime error consequatur cumque quaerat. Pariatur voluptatem dolorum ullam libero ut perspiciatis.
";

fn main() -> Result<(), Box<dyn Error>> {
    let mut window = Window::create(&WindowSettings::default())?;
    let mut renderer = Renderer::new(&window);
    let mut text = TextRenderer::create(&mut renderer);
    let bgs = renderer.create_draw_call(DrawCallParameters {
        alpha_blending: false,
        ..Default::default()
    });

    let mut time = 0.0f32;
    let mut was_mouse_in = vec![false; 3];
    let mut pressed_index = None;
    let mut lipsum_alignment = Alignment::Left;
    while window.refresh() {
        text.update_dpi_factor(window.dpi_factor);

        time += 0.01;
        let osc = time.sin() * 0.5 + 0.5;
        let mut y = 10.0;

        if let Some(rect) = text.draw_text(
            "First test, no limits, should be on one line.",
            (10.0, y, 0.0),
            16.0,
            Alignment::Left,
            (0.0, 0.0, 0.0, 1.0),
            None,
            None,
        ) {
            renderer.draw_quad_tinted(rect, (0.9, 0.9, 0.5, 1.0), (0.0, 0.0, 0.0), 0.1, &bgs);
        }
        y += 20.0;

        if let Some(rect) = text.draw_text(
            "Cut off at |, like so |...and here's text that should not appear",
            (10.0, y, 0.0),
            16.0,
            Alignment::Left,
            (0.0, 0.0, 0.0, 1.0),
            None,
            Some((10.0, y, 10.0 + 12.0 * 8.0, y + 16.0)),
        ) {
            renderer.draw_quad_tinted(rect, (0.9, 0.9, 0.5, 1.0), (0.0, 0.0, 0.0), 0.1, &bgs);
        }
        y += 20.0;

        {
            // Buttons
            let mouse_in = is_mouse_in(&window, (10.0, y, 210.0, y + 40.0));
            if mouse_in && !was_mouse_in[0] {
                window.set_cursor(fae::glutin::MouseCursor::Hand);
            } else if !mouse_in && was_mouse_in[0] {
                window.set_cursor(fae::glutin::MouseCursor::Default);
            }
            was_mouse_in[0] = mouse_in;
            renderer.draw_quad_tinted(
                (11.0, y + 1.0, 209.0, y + 39.0),
                if mouse_in {
                    (0.9, 0.9, 0.9, 1.0)
                } else {
                    (1.0, 1.0, 1.0, 1.0)
                },
                (0.0, 0.0, 0.0),
                0.1,
                &bgs,
            );
            renderer.draw_quad_tinted(
                (10.0, y, 210.0, y + 40.0),
                (0.2, 0.2, 0.2, 1.0),
                (0.0, 0.0, 0.0),
                0.1,
                &bgs,
            );
            text.draw_text(
                "Left",
                (10.0, y + 10.0, 0.0),
                20.0,
                Alignment::Left,
                (0.0, 0.0, 0.0, 1.0),
                Some(200.0),
                None,
            );
            y += 50.0;

            let mouse_in = is_mouse_in(&window, (10.0, y, 210.0, y + 40.0));
            if mouse_in && !was_mouse_in[1] {
                window.set_cursor(fae::glutin::MouseCursor::Hand);
            } else if !mouse_in && was_mouse_in[1] {
                window.set_cursor(fae::glutin::MouseCursor::Default);
            }
            was_mouse_in[1] = mouse_in;
            renderer.draw_quad_tinted(
                (11.0, y + 1.0, 209.0, y + 39.0),
                if mouse_in {
                    (0.9, 0.9, 0.9, 1.0)
                } else {
                    (1.0, 1.0, 1.0, 1.0)
                },
                (0.0, 0.0, 0.0),
                0.1,
                &bgs,
            );
            renderer.draw_quad_tinted(
                (10.0, y, 210.0, y + 40.0),
                (0.2, 0.2, 0.2, 1.0),
                (0.0, 0.0, 0.0),
                0.1,
                &bgs,
            );
            text.draw_text(
                "Center",
                (10.0, y + 10.0, 0.0),
                20.0,
                Alignment::Center,
                (0.0, 0.0, 0.0, 1.0),
                Some(200.0),
                None,
            );
            y += 50.0;

            let mouse_in = is_mouse_in(&window, (10.0, y, 210.0, y + 40.0));
            if mouse_in && !was_mouse_in[2] {
                window.set_cursor(fae::glutin::MouseCursor::Hand);
            } else if !mouse_in && was_mouse_in[2] {
                window.set_cursor(fae::glutin::MouseCursor::Default);
            }
            was_mouse_in[2] = mouse_in;
            renderer.draw_quad_tinted(
                (11.0, y + 1.0, 209.0, y + 39.0),
                if mouse_in {
                    (0.9, 0.9, 0.9, 1.0)
                } else {
                    (1.0, 1.0, 1.0, 1.0)
                },
                (0.0, 0.0, 0.0),
                0.1,
                &bgs,
            );
            renderer.draw_quad_tinted(
                (10.0, y, 210.0, y + 40.0),
                (0.2, 0.2, 0.2, 1.0),
                (0.0, 0.0, 0.0),
                0.1,
                &bgs,
            );
            text.draw_text(
                "Right",
                (10.0, y + 10.0, 0.0),
                20.0,
                Alignment::Right,
                (0.0, 0.0, 0.0, 1.0),
                Some(200.0),
                None,
            );
            y += 50.0;

            if window.mouse_pressed.contains(&Mouse::Left) {
                for i in 0..was_mouse_in.len() {
                    if was_mouse_in[i] {
                        pressed_index = Some(i);
                    }
                }
            }

            if window.mouse_released.contains(&Mouse::Left) {
                if let Some(i) = pressed_index {
                    if was_mouse_in[i] {
                        match i {
                            0 => lipsum_alignment = Alignment::Left,
                            1 => lipsum_alignment = Alignment::Center,
                            2 => lipsum_alignment = Alignment::Right,
                            _ => {}
                        }
                    }
                }
                pressed_index = None;
            }
            // Buttons
        }

        {
            // Animated text
            if let Some(rect) = text.draw_text(
                "FirstSecndThirdNow break at word boundary hah ha ha ha",
                (10.0, y, 0.0),
                16.0,
                Alignment::Left,
                (0.0, 0.0, 0.0, 1.0),
                Some(40.0 + osc * 100.0),
                None,
            ) {
                renderer.draw_quad_tinted(rect, (0.9, 0.9, 0.5, 1.0), (0.0, 0.0, 0.0), 0.1, &bgs);
                // y += 10.0 + (rect.3 - rect.1); // uncomment this if adding new y-based things
            }
            // Animated text
        }

        {
            // Lorem ipsum
            text.draw_text(
                LOREM_IPSUM,
                (300.0, 40.0, 0.0),
                16.0,
                lipsum_alignment,
                (0.0, 0.0, 0.0, 1.0),
                Some(320.0),
                None,
            );
            // Lorem ipsum
        }

        text.compose_draw_call(&mut renderer);
        renderer.render(window.width, window.height);
        window.swap_buffers(Some(&renderer));
    }
    Ok(())
}

fn is_mouse_in(window: &Window, rect: (f32, f32, f32, f32)) -> bool {
    let (x, y) = window.mouse_coords;
    x >= rect.0 && x <= rect.2 && y >= rect.1 && y <= rect.3
}
