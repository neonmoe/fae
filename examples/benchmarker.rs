// TODO(0.5.0): Rewrite benchmarker.rs after other todos.

//! This is less of an example and more of a benchmark.
//!
//! Note: this example will be pretty much entirely rewritten once
//! some API has been decided on. Currently it's just doing the calls
//! as they happen to be ordered in the codebase.
mod common;

use common::WindowSettings;
use fae::glutin::event_loop::ControlFlow;
use fae::{DrawCallParameters, GraphicsContext, Image, Window};
use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::create(WindowSettings::default().into()).unwrap();
    let mut text = common::create_font(&mut window.ctx);
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
    let call = window.ctx.create_draw_call(params);

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
    let mut time_ticking = false;
    // For testing text input
    let mut customizable_text = String::new();
    let mut last_over_text = false;

    // Mouse coords
    let (mut mx, mut my) = (0.0, 0.0);

    timers["whole frame"][timer_index].start();
    window.run(move |ctx, event, _, control_flow| {
        timers["whole frame"][timer_index].end();

        if should_quit {
            *control_flow = ControlFlow::Exit;
            return;
        }

        timer_index += 1;
        if timer_index >= max_timers {
            timer_index = 0;
        }

        timers["whole frame"][timer_index].start();
        timers["application frame"][timer_index].start();

        if let Some(mut ctx) = ctx {
            let over_text = mx > 200.0 && mx < 340.0 && my > 25.0 && my < 100.0;
            if over_text != last_over_text {
                //set_cursor_over_text(&mut ctx, over_text);
            }
            last_over_text = over_text;

            let time = Instant::now() - start;
            let time = if time_ticking {
                time.as_secs() as f32 + time.subsec_millis() as f32 / 1000.0
            } else {
                0.0
            };

            timers["quad calls"][timer_index].start();
            // Background
            call.draw(&mut ctx, -0.6)
                .with_coordinates((0.0, 0.0, 640.0, 480.0))
                .with_texture_coordinates((0, 0, 1240, 920))
                .finish();

            // Bottom right corned (for testing smooth resize)
            let (w, h) = (ctx.width, ctx.height);
            call.draw(&mut ctx, -0.5)
                .with_coordinates((w - 100.0, h - 100.0, 100.0, 100.0))
                .with_texture_coordinates((0, 0, 1240, 920))
                .finish();

            // Spinny sprites
            for i in 0..quad_count {
                let f = i as f32 / quad_count as f32;
                let x = (f * 3.1415 * 8.0 + time).cos() * 150.0 * f.max(0.3) + 270.0;
                let y = (f * 3.1415 * 8.0 + time).sin() * 150.0 * f.max(0.3) + 190.0;
                call.draw(&mut ctx, 0.5 - f)
                    .with_coordinates((x, y, 124.0, 92.0))
                    .with_texture_coordinates((0, 0, 1240, 920))
                    .with_color((1.0, 0.7, 0.9, 1.0))
                    .with_rotation(-time * 1.5, 50.0, 50.0)
                    .finish();
            }
            timers["quad calls"][timer_index].end();

            timers["text calls"][timer_index].start();
            // Draw some text describing the frame timings
            let text_color = (1.0, 0.0, 0.0, 1.0);
            let mut y = 5.0;

            for duration_name in &timer_names {
                y += 20.0;
                let s = format!(
                    "{}: {:4.1} \u{03bc}s",
                    duration_name,
                    get_avg_timer_mcs(duration_name)
                );
                text.draw(&mut ctx, s, 10.0, y, 0.6, 11.0)
                    .with_color(text_color)
                    .finish();
            }

            if let Some(rect) = text
                .draw(&mut ctx, "Wee!", 320.0, 340.0, 0.6, 14.0)
                .with_visibility(false)
                .finish()
            {
                let (half_width, half_height) = (rect.width / 2.0, rect.height / 2.0);
                let (x, y) = (320.0 - half_width, 340.0 - half_height);
                text.draw(&mut ctx, "Wee!", x, y, 0.6, 14.0)
                    .with_color((0.1, 0.2, 0.5, 1.0))
                    .with_rotation(time * 10.0, half_width, half_height)
                    .finish();
            }

            y += 20.0;
            let s = "Press Space to make everything spin. To demonstrate animation.";
            text.draw(&mut ctx, s, 10.0, y, 0.6, 11.0)
                .with_color(text_color)
                .finish();

            y += 20.0;
            let s = format!("Quad count: {}", quad_count);
            text.draw(&mut ctx, s, 10.0, y, 0.6, 11.0)
                .with_color(text_color)
                .finish();

            /*
            y += 20.0;
            let s = format!("Pressed keys: {:?}", window.held_keys);
            text.draw(&mut ctx, s, 10.0, y, 0.6, 11.0)
                .with_color(text_color)
                .finish();

            y += 20.0;
            let s = format!("Scaling factor: {:.1}", window.dpi_factor);
            text.draw(&mut ctx, s, 10.0, y, 0.6, 11.0)
                .with_color(text_color)
                .finish();

            y = 5.0;

            y += 20.0;
            let s = format!("Type some text: {}", customizable_text);
            text.draw(&mut ctx, s, 200.0, y, 0.6, 11.0)
                .with_color(text_color)
                .finish();

            y += 20.0;
            let s = format!("Mouse held: {:?}", window.mouse_held);
            text.draw(&mut ctx, s, 200.0, y, 0.6, 11.0)
                .with_color(text_color)
                .finish();

            y += 20.0;
            let (mouse_x, mouse_y) = window.mouse_coords;
            let s = format!("Mouse position: {}, {}", mouse_x, mouse_y);
            text.draw(&mut ctx, s, 200.0, y, 0.6, 11.0)
                .with_color(text_color)
                .finish();

            y += 20.0;
            let s = format!("Mouse in window: {}", window.mouse_inside);
            text.draw(&mut ctx, s, 200.0, y, 0.6, 11.0)
                .with_color(text_color)
                .finish();
            */

            y += 10.0;

            if let fae::gl_version::OpenGlVersion::Available { major, minor, .. } =
                ctx.get_opengl_version()
            {
                y += 20.0;
                let s = format!("OpenGL version: {}.{}", major, minor);
                text.draw(&mut ctx, s, 200.0, y, 0.6, 11.0)
                    .with_color(text_color)
                    .finish();
            }

            y += 20.0;
            let s = format!("OpenGL 3.3+ optimizations: {}", !ctx.is_legacy());
            text.draw(&mut ctx, s, 200.0, y, 0.6, 11.0)
                .with_color(text_color)
                .finish();

            #[cfg(feature = "profiler")]
            {
                let s = format!("{:#?}", fae::profiler::read());
                if let Some(mut rect) = text
                    .draw(&mut ctx, 30.0, 310.0, 0.6, 11.0)
                    .with_color(text_color)
                    .with_clip_area((20.0, 300.0, 420.0, 600.0))
                    .finish()
                {
                    rect.width = 380.0;
                    call.draw(&mut ctx, 0.55).with_coordinates(rect).finish();
                }
            }

            timers["text calls"][timer_index].end();
        } else {
            match event {
                /*
                    if window
                    .pressed_keys
                    .contains(&glutin::VirtualKeyCode::Escape)
                    {
                    should_quit = true;
                }
                    if window.pressed_keys.contains(&glutin::VirtualKeyCode::Up) {
                    quad_count *= 2;
                }
                    if window.pressed_keys.contains(&glutin::VirtualKeyCode::Down) && quad_count > 1 {
                    quad_count /= 2;
                }
                    if window.pressed_keys.contains(&glutin::VirtualKeyCode::Space) {
                    time_ticking = !time_ticking;
                }

                    for c in &window.typed_chars {
                    if !c.is_control() {
                    customizable_text.push(*c);
                }
                }
                     */
                _ => {}
            }
        }
        timers["application frame"][timer_index].end();
    });
}
/*
fn set_cursor_over_text(ctx: &GraphicsContext, over_text: bool) {
    use fae::glutin::window::CursorIcon;
    ctx.window().set_cursor_icon(if over_text {
        CursorIcon::Text
    } else {
        CursorIcon::Default
    });
}
*/
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

    pub fn start(&self) {
        self.start.set(Instant::now());
    }

    pub fn end(&self) {
        self.duration.set(Instant::now() - self.start.get());
    }

    pub fn duration(&self) -> f32 {
        self.duration.get().as_secs() as f32
            + (self.duration.get().subsec_nanos() as f64 / 1_000_000_000.0) as f32
    }
}
