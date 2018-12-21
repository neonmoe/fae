//! This is less of an example and more of a benchmark.
//!
//! Note: this example will be pretty much entirely rewritten once
//! some API has been decided on. Currently it's just doing the calls
//! as they happen to be ordered in the codebase.
#![windows_subsystem = "windows"]

use fae::{
    renderer::{DrawCallParameters, Renderer},
    text::{Alignment, TextRenderer},
    window::{Window, WindowSettings},
    Image,
};
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Create the window
    let mut window = Window::create(&WindowSettings::default()).unwrap();
    // Create the OpenGL renderer
    let mut renderer = Renderer::create(window.opengl21)?;
    // Create the text renderer
    let mut text = TextRenderer::create(fs::read("examples/res/FiraSans.ttf")?, &mut renderer)?;
    // Create the draw call for the sprite
    let params = DrawCallParameters {
        image: Some(Image::from_png(&fs::read("examples/res/sprite.png")?)?),
        ..Default::default()
    };
    let call = renderer.create_draw_call(params);

    let mut should_quit = false;
    let mut quad_count = 1;

    use std::time::Instant;
    let start = Instant::now();
    let mut frame_boundary = Instant::now();

    // Loop until we `should_quit` or refresh returns false, ie. the
    // user pressed the "close window" button.
    while window.refresh(|event| {
        // Handle events, as they are currently handled. This system
        // should really be revamped.
        if let glutin::Event::WindowEvent { event, .. } = event {
            match event {
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        if input.state == glutin::ElementState::Pressed {
                            match keycode {
                                glutin::VirtualKeyCode::Escape => should_quit = true,
                                glutin::VirtualKeyCode::Up => quad_count *= 2,
                                glutin::VirtualKeyCode::Down => {
                                    if quad_count > 1 {
                                        quad_count /= 2;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }) && !should_quit
    {
        // Update the text renderer's dpi settings, in case refresh
        // changed them
        text.update_dpi_factor(window.dpi_factor);

        let time = Instant::now() - start;
        let time = time.as_secs() as f32 + time.subsec_millis() as f32 / 1000.0;

        let start_quads = Instant::now();
        renderer.draw_quad(
            (0.0, 0.0, 640.0, 480.0),
            Some((0.0, 0.0, 1.0, 1.0)),
            (0xFF, 0xFF, 0xFF, 0xFF),
            (0.0, 0.0, 0.0),
            0.6,
            call,
        );
        // Draw a tinted sprite
        for i in 0..quad_count {
            let f = i as f32 / quad_count as f32;
            let x = (f * 3.1415 * 8.0 + time).cos() * 150.0 * f.max(0.3) + 270.0;
            let y = (f * 3.1415 * 8.0 + time).sin() * 150.0 * f.max(0.3) + 190.0;
            renderer.draw_quad(
                (x, y, x + 100.0, y + 100.0),
                Some((0.0, 0.0, 1.0, 1.0)),
                (0xFF, 0xAA, 0xEE, 0xFF),
                (-time * 1.5, 0.5, 0.5),
                0.5,
                call,
            );
        }
        let draw_quads_duration = Instant::now() - start_quads;
        let frame_time = Instant::now() - frame_boundary;
        frame_boundary = Instant::now();

        // Draw some text describing the frame timings
        let mut y = 20.0;
        text.draw_text(
            &format!("Frametime: {:?}", frame_time),
            (10.0, y, -0.5),
            16.0,
            Alignment::Left,
            None,
            None,
        );

        y += 20.0;
        text.draw_text(
            &format!("Quadtime: {:?}", draw_quads_duration),
            (10.0, y, -0.5),
            16.0,
            Alignment::Left,
            None,
            None,
        );

        y += 20.0;
        text.draw_text(
            &format!(
                "Quadtime of frametime: {:.1} %",
                draw_quads_duration.subsec_micros() as f64 / frame_time.subsec_micros() as f64
                    * 100.0
            ),
            (10.0, y, -0.5),
            16.0,
            Alignment::Left,
            None,
            None,
        );

        y += 20.0;
        text.draw_text(
            &format!("Quad count: {}", quad_count),
            (10.0, y, -0.5),
            16.0,
            Alignment::Left,
            None,
            None,
        );

        // Render the glyphs into the draw call
        text.compose_draw_call(&mut renderer);
        // Render the OpenGL draw calls
        renderer.render(window.width, window.height);
    }

    Ok(())
}
