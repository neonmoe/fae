//! This is less of an example and more of a benchmark.
//!
//! Note: this example will be pretty much entirely rewritten once
//! some API has been decided on. Currently it's just doing the calls
//! as they happen to be ordered in the codebase.
#![windows_subsystem = "windows"]

use fae::{
    renderer::{DrawCallParameters, Renderer},
    text::{Alignment, TextRenderer},
    Image,
};
use glfw::{self, Action, Context, Key};
use std::env;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

    let (mut window, events) = {
        glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
        glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        if let Some(result) = glfw.create_window(640, 480, "bench", glfw::WindowMode::Windowed) {
            result
        } else {
            glfw.window_hint(glfw::WindowHint::ContextVersionMajor(2));
            glfw.window_hint(glfw::WindowHint::ContextVersionMinor(1));
            glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(false));
            glfw.window_hint(glfw::WindowHint::OpenGlProfile(
                glfw::OpenGlProfileHint::Any,
            ));
            glfw.create_window(640, 480, "bench", glfw::WindowMode::Windowed)
                .expect("Failed to create GLFW window.")
        }
    };

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    glfw.make_context_current(Some(&window));
    fae::gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    if glfw.extension_supported("WGL_EXT_swap_control_tear")
        || glfw.extension_supported("GLX_EXT_swap_control_tear")
    {
        glfw.set_swap_interval(glfw::SwapInterval::Adaptive);
    } else {
        glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    }

    // Create the OpenGL renderer
    let mut renderer = Renderer::create(window.get_context_version().major == 2)?;
    // Create the text renderer
    let mut text = TextRenderer::create(fs::read("examples/res/FiraSans.ttf")?, &mut renderer)?;
    // Create the draw call for the sprite
    let params = DrawCallParameters {
        image: Some(Image::from_png(&fs::read("examples/res/sprite.png")?)?),
        ..Default::default()
    };
    let call = renderer.create_draw_call(params);

    // Get the most common DE's dpi factor env variables
    let dpi_factor = env::var("QT_SCALE_FACTOR")
        .or(env::var("GDK_SCALE"))
        .or(env::var("ELM_SCALE"))
        .ok()
        .and_then(|f| f.parse::<f32>().ok())
        .unwrap_or(1.0);
    text.update_dpi_factor(dpi_factor);

    let mut quad_count = 1;

    use std::time::Instant;
    let start = Instant::now();
    let mut frame_boundary = Instant::now();

    // Loop until we `should_quit` or refresh returns false, ie. the
    // user pressed the "close window" button.
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut quad_count);
        }

        let time = Instant::now() - start;
        let time = time.as_secs() as f32 + time.subsec_millis() as f32 / 1000.0;

        let start_quads = Instant::now();
        // Draw a tinted sprite
        for i in 0..quad_count {
            let x = (i as f32 / quad_count as f32 * 3.1415 * 2.0 + time).cos() * 140.0 + 160.0;
            let y = (i as f32 / quad_count as f32 * 3.1415 * 2.0 + time).sin() * 50.0 + 150.0;
            renderer.draw_quad(
                (x, y, x + 315.0, y + 235.0),
                Some((0.0, 0.0, 1.0, 1.0)),
                (0xFF, 0xAA, 0xEE, 0xFF),
                (0.0, 0.0, 0.0),
                0.5,
                call,
            );
        }
        let draw_quads_duration = Instant::now() - start_quads;
        let frame_time = Instant::now() - frame_boundary;
        frame_boundary = Instant::now();

        // Draw some text describing the frame timings
        text.draw_text(
            &format!("Frametime: {:?}", frame_time),
            (10.0, 20.0, -0.5),
            (200.0, 16.0),
            Alignment::Left,
            false,
        );
        text.draw_text(
            &format!("Quadtime: {:?}", draw_quads_duration),
            (10.0, 40.0, -0.5),
            (200.0, 16.0),
            Alignment::Left,
            false,
        );
        text.draw_text(
            &format!("Quad count: {}", quad_count),
            (10.0, 60.0, -0.5),
            (200.0, 16.0),
            Alignment::Left,
            false,
        );
        text.draw_text(
            &format!(
                "Quad VRAM usage (approx.): {:.1} KB",
                (quad_count * 24) as f32 / 1000.0
            ),
            (10.0, 80.0, -0.5),
            (400.0, 16.0),
            Alignment::Left,
            false,
        );

        // Render the glyphs into the draw call
        text.compose_draw_call(&mut renderer);
        // Render the OpenGL draw calls
        let (width, height) = window.get_size();
        renderer.render(width as f32 / dpi_factor, height as f32 / dpi_factor);
        window.swap_buffers();
    }

    Ok(())
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, count: &mut i32) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::Key(Key::Up, _, Action::Press, _) => *count *= 2,
        glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => {
            if *count > 1 {
                *count /= 2
            }
        }
        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            fae::gl::Viewport(0, 0, width, height)
        },
        _ => {}
    }
}
