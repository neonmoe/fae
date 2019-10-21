//! This is less of an example and more of a benchmark.
//!
//! Note: this example will be pretty much entirely rewritten once
//! some API has been decided on. Currently it's just doing the calls
//! as they happen to be ordered in the codebase.
#![windows_subsystem = "windows"]

use fae::text::{Alignment, TextRenderer};
use fae::{profiler, DrawCallParameters, Image, Renderer, Window, WindowSettings};
use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, Instant};

mod keys {
    pub const CLOSE: glutin::VirtualKeyCode = glutin::VirtualKeyCode::Escape;
    pub const UP: glutin::VirtualKeyCode = glutin::VirtualKeyCode::Up;
    pub const DOWN: glutin::VirtualKeyCode = glutin::VirtualKeyCode::Down;
    pub const PROFILE: glutin::VirtualKeyCode = glutin::VirtualKeyCode::R;
}

static mut PROFILING: bool = false;

fn main() -> Result<(), Box<dyn Error>> {
    // Create the window
    let mut window = Window::create(&WindowSettings::default()).unwrap();
    // Create the OpenGL renderer
    let mut renderer = Renderer::new(&window);
    renderer.preserve_gl_state = false;
    // Create the text renderer
    let mut text = TextRenderer::create(&mut renderer);
    // Create the draw call for the sprite
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

    let mut should_quit = false;
    let mut quad_count = 1;

    let timer_names = [
        "whole frame",
        "application frame",
        "opengl",
        "swap buffers",
        "glyph rendering",
        "quad calls",
        "text calls",
    ];
    let mut timer_index = 0;
    let max_timers = 60;
    let mut timers: HashMap<&'static str, Vec<Timer>> = HashMap::new();
    for name in &timer_names {
        timers.insert(name, vec![Timer::new(name.to_string()); max_timers]);
    }

    let get_avg_timer_mcs = |name: &str| {
        timers[name]
            .iter()
            .map(|timer: &Timer| timer.duration())
            .sum::<f32>()
            / max_timers as f32
            * 1_000_000.0
    };

    // For animations
    let start = Instant::now();
    // For testing text input
    let mut customizable_text = String::new();
    let mut last_over_text = false;

    // Loop until we `should_quit` or refresh returns false, ie. the
    // user pressed the "close window" button.
    timers["whole frame"][timer_index].start();
    while window.refresh() && !should_quit {
        timers["whole frame"][timer_index].end();

        timer_index += 1;
        if timer_index >= max_timers {
            timer_index = 0;
        }

        unsafe {
            PROFILING = window.pressed_keys.contains(&keys::PROFILE);
        }

        timers["whole frame"][timer_index].start();
        timers["application frame"][timer_index].start();
        // Update the text renderer's dpi settings, in case refresh
        // changed them
        text.update_dpi_factor(window.dpi_factor);

        if window.pressed_keys.contains(&keys::CLOSE) {
            should_quit = true;
        }
        if window.pressed_keys.contains(&keys::UP) {
            quad_count *= 2;
        }
        if window.pressed_keys.contains(&keys::DOWN) && quad_count > 1 {
            quad_count /= 2;
        }

        for c in &window.typed_chars {
            if !c.is_control() {
                customizable_text.push(*c);
            }
        }

        let (mx, my) = window.mouse_coords;
        let over_text = mx > 200.0 && mx < 340.0 && my > 25.0 && my < 100.0;
        if over_text != last_over_text {
            set_cursor_over_text(&mut window, over_text);
        }
        last_over_text = over_text;

        renderer.set_profiling(window.pressed_keys.contains(&keys::PROFILE));

        let time = Instant::now() - start;
        let time = time.as_secs() as f32 + time.subsec_millis() as f32 / 1000.0;

        timers["quad calls"][timer_index].start();
        // Background
        renderer.draw_quad(
            (0.0, 0.0, 640.0, 480.0),
            (0.0, 0.0, 1.0, 1.0),
            (1.0, 1.0, 1.0, 1.0),
            (0.0, 0.0, 0.0),
            0.6,
            &call,
        );

        // Bottom right corned (for testing smooth resize)
        let (w, h) = (window.width, window.height);
        renderer.draw_quad(
            (w - 100.0, h - 100.0, w, h),
            (0.0, 0.0, 1.0, 1.0),
            (1.0, 1.0, 0.0, 1.0),
            (0.0, 0.0, 0.0),
            0.5,
            &call,
        );

        // Spinny sprites
        for i in 0..quad_count {
            let f = i as f32 / quad_count as f32;
            let x = (f * 3.1415 * 8.0 + time).cos() * 150.0 * f.max(0.3) + 270.0;
            let y = (f * 3.1415 * 8.0 + time).sin() * 150.0 * f.max(0.3) + 190.0;
            renderer.draw_quad(
                (x, y, x + 100.0, y + 100.0),
                (0.0, 0.0, 1.0, 1.0),
                (1.0, 0.7, 0.9, 1.0),
                (-time * 1.5, 50.0, 50.0),
                f - 0.5,
                &call,
            );
        }
        timers["quad calls"][timer_index].end();

        timers["text calls"][timer_index].start();
        // Draw some text describing the frame timings
        let text_color = (1.0, 0.0, 0.0, 1.0);
        let mut y = 5.0;

        if cfg!(feature = "flame") {
            text.draw_text(
                "Press R to record a frame with flame, see results in flame-graph.html after exiting the application",
                (20.0, y, -0.6),
                16.0,
                Alignment::Left, text_color,
                None,
                None,
            );
        }

        for duration_name in &timer_names {
            y += 20.0;
            text.draw_text(
                &format!(
                    "{}: {:4.1} μs",
                    duration_name,
                    get_avg_timer_mcs(duration_name)
                ),
                (10.0, y, -0.6),
                16.0,
                Alignment::Left,
                text_color,
                None,
                None,
            );
        }

        y += 20.0;
        text.draw_text(
            &format!("Quad count: {}", quad_count),
            (10.0, y, -0.6),
            16.0,
            Alignment::Left,
            text_color,
            None,
            None,
        );

        y += 20.0;
        text.draw_text(
            &format!("Pressed keys: {:?}", window.held_keys),
            (10.0, y, -0.6),
            16.0,
            Alignment::Left,
            text_color,
            None,
            None,
        );

        y += 20.0;
        text.draw_text(
            &format!("Scaling factor: {:.1}", window.dpi_factor),
            (10.0, y, -0.6),
            16.0,
            Alignment::Left,
            text_color,
            None,
            None,
        );

        y = 5.0;

        y += 20.0;
        text.draw_text(
            &format!("Type some text: {}", customizable_text),
            (200.0, y, -0.6),
            16.0,
            Alignment::Left,
            text_color,
            None,
            None,
        );

        y += 20.0;
        text.draw_text(
            &format!("Mouse held: {:?}", window.mouse_held),
            (200.0, y, -0.6),
            16.0,
            Alignment::Left,
            text_color,
            None,
            None,
        );

        y += 20.0;
        let (mouse_x, mouse_y) = window.mouse_coords;
        text.draw_text(
            &format!("Mouse position: {}, {}", mouse_x, mouse_y),
            (200.0, y, -0.6),
            16.0,
            Alignment::Left,
            text_color,
            None,
            None,
        );

        y += 20.0;
        text.draw_text(
            &format!("Mouse in window: {}", window.mouse_inside),
            (200.0, y, -0.6),
            16.0,
            Alignment::Left,
            text_color,
            None,
            None,
        );

        y += 10.0;

        if let Some((major_ver, minor_ver)) = renderer.get_opengl_version() {
            y += 20.0;
            text.draw_text(
                &format!("OpenGL version: {}.{}", major_ver, minor_ver),
                (200.0, y, -0.6),
                16.0,
                Alignment::Left,
                text_color,
                None,
                None,
            );
        }

        y += 20.0;
        text.draw_text(
            &format!("OpenGL 3.3+ optimizations: {}", !renderer.is_legacy()),
            (200.0, y, -0.6),
            16.0,
            Alignment::Left,
            text_color,
            None,
            None,
        );

        renderer.draw_quad(
            (20.0, 300.0, 320.0, 600.0),
            (-1.0, -1.0, -1.0, -1.0),
            (1.0, 1.0, 1.0, 1.0),
            (0.0, 0.0, 0.0),
            -0.55,
            &call,
        );
        text.draw_text(
            &format!("profiling information:\n{}", profiler::get_profiler_print()),
            (30.0, 310.0, -0.6),
            16.0,
            Alignment::Left,
            text_color,
            Some(280.0),
            Some((20.0, 300.0, 320.0, 600.0)),
        );

        timers["text calls"][timer_index].end();

        // Render the glyphs into the draw call
        timers["glyph rendering"][timer_index].start();
        text.compose_draw_call(&mut renderer);
        timers["glyph rendering"][timer_index].end();
        // Render the OpenGL draw calls
        timers["opengl"][timer_index].start();
        renderer.render(window.width, window.height);
        timers["opengl"][timer_index].end();
        timers["swap buffers"][timer_index].start();
        window.swap_buffers(Some(&renderer));
        timers["swap buffers"][timer_index].end();

        timers["application frame"][timer_index].end();
    }

    dump_profiling_data();

    Ok(())
}

fn set_cursor_over_text(window: &Window, over_text: bool) {
    use fae::glutin::MouseCursor;
    window.set_cursor(if over_text {
        MouseCursor::Text
    } else {
        MouseCursor::Default
    });
}

#[cfg(feature = "flame")]
fn dump_profiling_data() {
    use std::fs::File;
    flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
}

#[cfg(not(feature = "flame"))]
fn dump_profiling_data() {}

use std::cell::Cell;

#[derive(Debug, Clone)]
struct Timer {
    start: Cell<Instant>,
    duration: Cell<Duration>,
    name: String,
}

impl Timer {
    pub fn new(name: String) -> Timer {
        Timer {
            start: Cell::new(Instant::now()),
            duration: Cell::new(Duration::from_secs(0)),
            name,
        }
    }

    #[cfg(feature = "flame")]
    pub fn start(&self) {
        self.start.set(Instant::now());
        if unsafe { PROFILING } {
            flame::start(self.name.clone());
        }
    }

    #[cfg(feature = "flame")]
    pub fn end(&self) {
        if unsafe { PROFILING } {
            flame::end(self.name.clone());
        }
        self.duration.set(Instant::now() - self.start.get());
    }

    #[cfg(not(feature = "flame"))]
    pub fn start(&self) {
        self.start.set(Instant::now());
    }

    #[cfg(not(feature = "flame"))]
    pub fn end(&self) {
        self.duration.set(Instant::now() - self.start.get());
    }

    pub fn duration(&self) -> f32 {
        self.duration.get().as_secs() as f32
            + (self.duration.get().subsec_nanos() as f64 / 1_000_000_000.0) as f32
    }
}
