//! This example writes text in varying ways to test that the layout
//! functionality works correctly.

#[cfg(feature = "font8x8")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use fae::{
        text::{self, Alignment, TextRenderer},
        Renderer, Window, WindowSettings,
    };

    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::create(&WindowSettings::default())?;
    let mut renderer = Renderer::new(&window);
    let mut text = TextRenderer::with_font8x8(&mut renderer, true);
    let call = renderer.create_draw_call(Default::default());

    let mut offset = 0;
    let len = 100;
    let mut all_characters = String::new();
    for u in 0..0xFFFF {
        if text::get_bitmap(u).is_some() {
            use std::convert::TryFrom;
            if let Ok(c) = char::try_from(u) {
                all_characters.push(c);
            }
        }
    }

    while window.refresh() {
        renderer.set_dpi_factor(window.dpi_factor);
        text.set_dpi_factor(window.dpi_factor);

        offset += 1;
        let s: String = all_characters
            .chars()
            .cycle()
            .skip(offset)
            .take(len)
            .collect();

        text.draw_text(
            &s,
            (10.0, 30.0, 0.0),
            12.0,
            Alignment::Left,
            (0.0, 0.0, 0.0, 1.0),
            Some(window.width - 20.0),
            None,
        );

        text.draw_text(
            "Every character in font8x8:",
            (10.0, 10.0, 0.0),
            10.0,
            Alignment::Left,
            (0.0, 0.0, 0.0, 1.0),
            None,
            None,
        );

        let profiling_data = fae::profiler::read();
        text.draw_text(
            &format!("{:#?}", profiling_data),
            (10.0, 340.0, 0.0),
            10.0,
            Alignment::Left,
            (0.0, 0.0, 0.0, 1.0),
            None,
            None,
        );

        let misses = profiling_data.glyph_cache_misses;
        let total = profiling_data.glyphs_drawn;
        if total > 0 {
            text.draw_text(
                &format!(
                    "Glyph cache miss ratio: {:3.1} %",
                    (misses as f32 / total as f32 * 100.0)
                ),
                (10.0, 310.0, 0.0),
                10.0,
                Alignment::Left,
                (0.0, 0.0, 0.0, 1.0),
                None,
                None,
            );
        }

        let cache_size = 256.0 / window.dpi_factor;
        let (x, y) = (
            window.width as f32 - 20.0 - cache_size,
            window.height as f32 - 20.0 - cache_size,
        );
        text.debug_draw_glyph_cache(&mut renderer, (x, y, cache_size, cache_size), -0.8);
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

#[cfg(not(feature = "font8x8"))]
fn main() {
    log::error!("font8x8 feature is required for the `font8x8_glyphs` example");
}
