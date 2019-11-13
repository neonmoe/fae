//! This example writes text in varying ways to test that the layout
//! functionality works correctly.

#[cfg(feature = "font8x8")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use fae::{
        profiler,
        text::{self, Alignment, TextRenderer},
        Renderer, Window, WindowSettings,
    };

    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::create(&WindowSettings::default())?;
    let mut renderer = Renderer::new(&window);
    let mut text = TextRenderer::with_font8x8(&mut renderer, true);
    let call = renderer.create_draw_call(Default::default());

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

        text.draw_text(
            &all_characters,
            (10.0, 30.0, 0.0),
            12.0,
            Alignment::Left,
            (0.0, 0.0, 0.0, 1.0),
            Some(window.width - 20.0),
            None,
        );

        text.draw_text(
            "Every character in font8x8, 32 per row:",
            (10.0, 10.0, 0.0),
            10.0,
            Alignment::Left,
            (0.0, 0.0, 0.0, 1.0),
            None,
            None,
        );

        text.draw_text(
            &profiler::get_profiler_print(),
            (10.0, 340.0, 0.0),
            10.0,
            Alignment::Left,
            (0.0, 0.0, 0.0, 1.0),
            None,
            None,
        );

        let cache_size = 256.0 / window.dpi_factor;
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

#[cfg(not(feature = "font8x8"))]
fn main() {
    log::error!("font8x8 feature is required for the `font8x8_glyphs` example");
}
