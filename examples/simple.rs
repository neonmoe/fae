//! This is a very simple example, mostly for profiling frame times.
//!
//! Note: this example will be pretty much entirely rewritten once
//! some API has been decided on. Currently it's just doing the calls
//! as they happen to be ordered in the codebase.
#![windows_subsystem = "windows"]

use fae::{
    renderer::Renderer,
    text::{Alignment, TextRenderer},
    window::{Window, WindowSettings},
    Image,
};
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Create the window
    let mut window = Window::create(WindowSettings::default()).unwrap();
    // Create the OpenGL renderer
    let mut renderer = Renderer::create(window.opengl21)?;
    // Create the text renderer
    let mut text = TextRenderer::create(fs::read("examples/res/FiraSans.ttf")?, &mut renderer)?;
    // Create the draw call for the sprite
    let call = renderer.create_draw_call(
        &Image::from_png(&fs::read("examples/res/sprite.png")?)?,
        None,
    );

    let mut should_quit = false;

    // Loop until we `should_quit` or refresh returns false, ie. the
    // user pressed the "close window" button.
    while window.refresh(|event| {
        // Handle events, as they are currently handled. This system
        // should really be revamped.
        if let glutin::Event::WindowEvent { event, .. } = event {
            match event {
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == glutin::ElementState::Pressed {
                        if let Some(keycode) = input.virtual_keycode {
                            should_quit = keycode == glutin::VirtualKeyCode::Escape;
                        }
                    }
                }
                _ => {}
            }
        }
    }) && !should_quit
    {
        let frame_timer = &window.frame_timer;

        // Draw a tinted sprite
        renderer.draw_quad(
            (10.0, 10.0, 630.0, 470.0),
            (0.0, 0.0, 1.0, 1.0),
            (0xFF, 0xAA, 0xEE, 0xFF),
            0.5,
            call,
        );

        // Update the text renderer's dpi settings, in case refresh
        // changed them
        text.update_dpi(window.dpi);
        // Draw some text describing the frame timings
        text.queue_text(
            &format!(
                "{} Hz, {:?}",
                frame_timer.frames_last_second(),
                frame_timer.avg_frame_duration(),
            ),
            (30.0, 30.0, -0.5),
            (200.0, 16.0),
            Alignment::Left,
            false,
        );

        // Render the glyphs into the draw call
        text.draw_text(&mut renderer);
        // Render the OpenGL draw calls
        renderer.render(window.width, window.height);
    }

    Ok(())
}
