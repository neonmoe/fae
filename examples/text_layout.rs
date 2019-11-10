//! This example writes text in varying ways to test that the layout
//! functionality works correctly.

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
    env_logger::init();

    let mut window = Window::create(&WindowSettings::default())?;
    let mut renderer = Renderer::new(&window);
    let mut text = TextRenderer::with_font8x8(&mut renderer, true);
    let bgs = renderer.create_draw_call(DrawCallParameters {
        alpha_blending: false,
        ..Default::default()
    });
    let call = renderer.create_draw_call(Default::default());

    let mut was_mouse_in = vec![false; 3];
    let mut pressed_index = None;
    let mut lipsum_alignment = Alignment::Left;
    while window.refresh() {
        renderer.set_dpi_factor(window.dpi_factor);
        text.set_dpi_factor(window.dpi_factor);

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
            renderer
                .draw(&bgs, 0.1)
                .with_coordinates(rect)
                .with_color(0.9, 0.9, 0.5, 1.0)
                .finish();
        }
        y += 20.0;

        if let Some(rect) = text.draw_text(
            "Cut off at |, like so |",
            (10.0, y, 0.0),
            14.0,
            Alignment::Left,
            (0.0, 0.0, 0.0, 1.0),
            None,
            None,
        ) {
            if let Some(rect) = text.draw_text(
                "Cut off at |, like so |...and here's text that should not appear",
                (10.0, y, 0.0),
                14.0,
                Alignment::Left,
                (0.0, 0.0, 0.0, 1.0),
                None,
                Some(rect),
            ) {
                renderer
                    .draw(&bgs, 0.1)
                    .with_coordinates(rect)
                    .with_color(0.9, 0.9, 0.5, 1.0)
                    .finish();
            }
        }
        y += 20.0;

        {
            // Buttons
            let px = 2.0;
            let mouse_in = is_mouse_in(&window, (10.0, y, 210.0, y + 40.0));
            if mouse_in && !was_mouse_in[0] {
                window
                    .get_window()
                    .set_cursor(fae::glutin::MouseCursor::Hand);
            } else if !mouse_in && was_mouse_in[0] {
                window
                    .get_window()
                    .set_cursor(fae::glutin::MouseCursor::Default);
            }
            was_mouse_in[0] = mouse_in;
            let col = if mouse_in { 0.9 } else { 1.0 };
            renderer
                .draw(&bgs, 0.1)
                .with_coordinates((10.0 + px, y + px, 200.0 - 2.0 * px, 40.0 - 2.0 * px))
                .with_color(col, col, col, 1.0)
                .finish();
            renderer
                .draw(&bgs, 0.1)
                .with_coordinates((10.0, y, 200.0, 40.0))
                .with_color(0.2, 0.2, 0.2, 1.0)
                .finish();
            text.draw_text(
                " Left",
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
                window
                    .get_window()
                    .set_cursor(fae::glutin::MouseCursor::Hand);
            } else if !mouse_in && was_mouse_in[1] {
                window
                    .get_window()
                    .set_cursor(fae::glutin::MouseCursor::Default);
            }
            was_mouse_in[1] = mouse_in;
            let col = if mouse_in { 0.9 } else { 1.0 };
            renderer
                .draw(&bgs, 0.1)
                .with_coordinates((10.0 + px, y + px, 200.0 - 2.0 * px, 40.0 - 2.0 * px))
                .with_color(col, col, col, 1.0)
                .finish();
            renderer
                .draw(&bgs, 0.1)
                .with_coordinates((10.0, y, 200.0, 40.0))
                .with_color(0.2, 0.2, 0.2, 1.0)
                .finish();
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
                window
                    .get_window()
                    .set_cursor(fae::glutin::MouseCursor::Hand);
            } else if !mouse_in && was_mouse_in[2] {
                window
                    .get_window()
                    .set_cursor(fae::glutin::MouseCursor::Default);
            }
            was_mouse_in[2] = mouse_in;
            let col = if mouse_in { 0.9 } else { 1.0 };
            renderer
                .draw(&bgs, 0.1)
                .with_coordinates((10.0 + px, y + px, 200.0 - 2.0 * px, 40.0 - 2.0 * px))
                .with_color(col, col, col, 1.0)
                .finish();
            renderer
                .draw(&bgs, 0.1)
                .with_coordinates((10.0, y, 200.0, 40.0))
                .with_color(0.2, 0.2, 0.2, 1.0)
                .finish();
            text.draw_text(
                "Right ",
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
            // Size comparisons
            for i in 0..8 {
                if let Some(rect) = text.draw_text(
                    "The quick brown fox jumps over the lazy dog",
                    (10.0, y, 0.0),
                    (8 + i) as f32,
                    Alignment::Left,
                    (0.0, 0.0, 0.0, 1.0),
                    None,
                    None,
                ) {
                    y += rect.height + 1.0;
                }
            }
            // Size comparisons
        }

        {
            // Lorem ipsum
            let font_size = 8.0;
            text.draw_text(
                &format!(
                    "Font size of lorem ipsum: {} px",
                    (font_size * window.dpi_factor) as i32
                ),
                (300.0, 30.0, 0.0),
                font_size / window.dpi_factor,
                lipsum_alignment,
                (0.1, 0.1, 0.1, 1.0),
                Some(320.0),
                None,
            );
            text.draw_text(
                LOREM_IPSUM,
                (300.0, 40.0, 0.0),
                font_size,
                lipsum_alignment,
                (0.0, 0.0, 0.0, 1.0),
                Some(320.0),
                None,
            );
            // Lorem ipsum
        }

        let cache_size = 128.0 / window.dpi_factor;
        let (x, y) = (
            window.width as f32 - 20.0 - cache_size,
            window.height as f32 - 20.0 - cache_size,
        );
        text.debug_draw_glyph_cache(&mut renderer, (x, y, x + cache_size, y + cache_size), -1.0);
        renderer
            .draw(&call, -0.9)
            .with_coordinates((x, y, cache_size, cache_size))
            .with_color(0.9, 0.9, 0.9, 1.0)
            .finish();

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
