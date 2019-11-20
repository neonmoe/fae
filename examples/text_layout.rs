//! This example writes text in varying ways to test that the layout
//! functionality works correctly.

mod common;

#[cfg(feature = "rusttype")]
use fae::{text::TextRenderer, Image};

use fae::{text::Alignment, DrawCallParameters, Mouse, Window, WindowSettings};
use std::error::Error;

static LOREM_IPSUM: &'static str = "Perferendis officiis ut provident sit eveniet ipsa eos. Facilis delectus at laudantium nemo. Sed ipsa natus perferendis dignissimos odio deserunt omnis.

Reprehenderit voluptas provident eveniet eos hic et maiores. Voluptatum totam sit quisquam consequatur atque sunt animi. Rem deleniti ex quia consequatur voluptate nostrum.

In earum architecto qui sunt provident. Vitae rerum molestiae dolorem praesentium fugit nostrum voluptas omnis. Rem sint voluptatem autem eum. Tempore velit maxime error consequatur cumque quaerat. Pariatur voluptatem dolorum ullam libero ut perspiciatis.
";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::create(&WindowSettings::default())?;
    let (mut renderer, mut text) = common::create_renderers(&window);

    #[cfg(feature = "rusttype")]
    let mut fira_sans =
        TextRenderer::with_ttf(&mut renderer, include_bytes!("res/FiraSans.ttf").to_vec()).unwrap();
    #[cfg(feature = "rusttype")]
    let sample_text = renderer.create_draw_call(DrawCallParameters {
        image: Some(Image::from_png(include_bytes!(
            "res/fira_sans_16px_sample.png"
        ))?),
        ..Default::default()
    });

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

        #[cfg(feature = "rusttype")]
        fira_sans.set_dpi_factor(window.dpi_factor);

        let mut y = 10.0;

        let s = "First test, no limits, should be on one line.";
        if let Some(rect) = text
            .draw(s, 10.0, y, 0.0, 16.0)
            .with_cacheable(true)
            .finish()
        {
            renderer
                .draw(&bgs, -0.1)
                .with_coordinates(rect)
                .with_color(0.9, 0.9, 0.5, 1.0)
                .finish();
        }
        y += 20.0;

        if let Some(rect) = text
            .draw("Cut off at |, like so |", 10.0, y, 0.0, 14.0)
            .with_cacheable(true)
            .finish()
        {
            let s = "Cut off at |, like so |...and here's text that should not appear";
            if let Some(rect) = text
                .draw(s, 10.0, y, 0.0, 14.0)
                .with_cacheable(true)
                .with_clip_area(rect)
                .finish()
            {
                renderer
                    .draw(&bgs, -0.1)
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
                .draw(&bgs, -0.1)
                .with_coordinates((10.0 + px, y + px, 200.0 - 2.0 * px, 40.0 - 2.0 * px))
                .with_color(col, col, col, 1.0)
                .finish();
            renderer
                .draw(&bgs, -0.1)
                .with_coordinates((10.0, y, 200.0, 40.0))
                .with_color(0.2, 0.2, 0.2, 1.0)
                .finish();
            text.draw("Left", 20.0, y + 10.0, 0.0, 20.0)
                .with_max_width(190.0)
                .with_alignment(Alignment::Left)
                .with_cacheable(true)
                .finish();
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
                .draw(&bgs, -0.1)
                .with_coordinates((10.0 + px, y + px, 200.0 - 2.0 * px, 40.0 - 2.0 * px))
                .with_color(col, col, col, 1.0)
                .finish();
            renderer
                .draw(&bgs, -0.1)
                .with_coordinates((10.0, y, 200.0, 40.0))
                .with_color(0.2, 0.2, 0.2, 1.0)
                .finish();
            text.draw("Center", 10.0, y + 10.0, 0.0, 20.0)
                .with_max_width(200.0)
                .with_alignment(Alignment::Center)
                .with_cacheable(true)
                .finish();
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
                .draw(&bgs, -0.1)
                .with_coordinates((10.0 + px, y + px, 200.0 - 2.0 * px, 40.0 - 2.0 * px))
                .with_color(col, col, col, 1.0)
                .finish();
            renderer
                .draw(&bgs, -0.1)
                .with_coordinates((10.0, y, 200.0, 40.0))
                .with_color(0.2, 0.2, 0.2, 1.0)
                .finish();
            text.draw("Right", 10.0, y + 10.0, 0.0, 20.0)
                .with_max_width(190.0)
                .with_alignment(Alignment::Right)
                .with_cacheable(true)
                .finish();
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
            for i in 0..12 {
                let s = "The quick brown fox jumps over the lazy dog";
                if let Some(rect) = text
                    .draw(s, 10.0, y, 0.0, (8 + i) as f32 / window.dpi_factor)
                    .with_cacheable(true)
                    .finish()
                {
                    y += rect.height + 1.0;
                }
            }
            // Size comparisons
        }

        {
            // Lorem ipsum
            let font_size = 11.0;
            let s = format!(
                "Font size of lorem ipsum: {} px",
                (font_size * window.dpi_factor) as i32
            );
            text.draw(s, 300.0, 30.0, 0.0, font_size / window.dpi_factor)
                .with_alignment(lipsum_alignment)
                .with_color((0.1, 0.1, 0.1, 1.0))
                .with_max_width(320.0)
                .with_cacheable(true)
                .finish();
            text.draw(LOREM_IPSUM, 300.0, 40.0, 0.0, font_size)
                .with_alignment(lipsum_alignment)
                .with_max_width(320.0)
                .with_cacheable(true)
                .finish();
            // Lorem ipsum
        }

        #[cfg(feature = "rusttype")]
        {
            // Fae/firefox text comparison
            // (it's not really fair, firefox does a lot more stuff, but as far as latin goes..)
            let font_size = 16.0 / window.dpi_factor;
            let mut y = window.height - 75.0;
            let x = 20.0;
            let comparison_x = 70.0;

            let s = "Comparison between text laid out by Fae and by Firefox:";
            text.draw(s, x, y, 0.0, 12.0).with_cacheable(true).finish();
            y += 20.0;

            let s = "Fae:";
            text.draw(s, x, y, 0.0, font_size)
                .with_cacheable(true)
                .finish();

            let s = "The quick brown fox jumps over the lazy dog.";
            fira_sans
                .draw(s, comparison_x + 1.5 / window.dpi_factor, y, 0.0, font_size)
                .with_cacheable(true)
                .finish();
            y += font_size * 1.25;

            let s = "Firefox:";
            text.draw(s, x, y, 0.0, font_size)
                .with_cacheable(true)
                .finish();
            renderer
                .draw(&sample_text, 0.0)
                .with_coordinates((
                    comparison_x,
                    y,
                    329.0 / window.dpi_factor,
                    19.0 / window.dpi_factor,
                ))
                .with_texture_coordinates((0, 0, 329, 19))
                .finish();
            // Fae/firefox text comparison
        }

        let cache_size = 256.0 / window.dpi_factor;
        let (x, y) = (
            window.width as f32 - 20.0 - cache_size,
            window.height as f32 - 20.0 - cache_size,
        );
        text.debug_draw_glyph_cache(&mut renderer, (x, y, cache_size, cache_size), 0.9);
        renderer
            .draw(&call, 0.8)
            .with_coordinates((x, y, cache_size, cache_size))
            .with_color(0.9, 0.9, 0.9, 1.0)
            .finish();

        #[cfg(feature = "rusttype")]
        fira_sans.compose_draw_call(&mut renderer);

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
