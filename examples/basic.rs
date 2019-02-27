//! This is a small example showing off most of the functionality of the crate.
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
    let mut renderer = Renderer::new(window.opengl21);
    renderer.preserve_gl_state = false;

    // Create the draw call for the sprite
    let params = DrawCallParameters {
        image: Some(Image::from_png(&fs::read("examples/res/sprite.png")?)?),
        alpha_blending: false,
        ..Default::default()
    };
    let call = renderer.create_draw_call(params);

    // Create the text renderer
    let mut text =
        TextRenderer::create(fs::read("examples/res/FiraSans.ttf")?, false, &mut renderer)?;

    // Loop until we `should_quit` or refresh returns false, ie. the
    // user pressed the "close window" key.
    let mut should_quit = false;
    while window.refresh() && !should_quit {
        // Update the text renderer's dpi settings, in case refresh
        // changed them, needs to be done before any text drawing and
        // after refresh() for correct results.
        text.update_dpi_factor(window.dpi_factor);

        // The pressed_keys Vec contains the keys which were pressed
        // down during this frame, use held_keys if you want to do
        // something every frame when a key is being pressed.
        if window.pressed_keys.contains(&keys::CLOSE) {
            should_quit = true;
        }

        // Draw a quad (filling the window if it hasn't been resized)
        renderer.draw_quad(
            (0.0, 0.0, 640.0, 480.0), // The corner coordinates of the quad, in window coordinates (x0, y0, x1, y1)
            (0.0, 0.0, 1.0, 1.0), // The corner texture coordinates of the quad, in the 0..1 range (x0, y0, x1, y1)
            (1.0, 1.0, 1.0, 1.0), // The tint color of the texture (r, g, b, a)
            (0.0, 0.0, 0.0), // The rotation and pivot offset (radians, x, y). If x = 0.0 and y = 0.0, the quad will be rotated around its top-left coordinate, and this is shifted by x and y
            0.5, // The z coordinate of the quad, to specify which goes in front of what. Negative values are in front.
            &call, // The draw call during which to render this quad, practically this decides which texture is used
        );

        // Some text
        text.draw_text(
            "Some cool text!",  // The displayed text
            (10.0, 10.0, -0.6), // The position (x, y, z)
            16.0,               // The font size
            Alignment::Left,    // The text alignment (only applied if max_row_width is specified)
            None,               // The maximum width of a row
            None,               // The clipping area, if text overflows this, it gets cut off
        );

        // Render the glyphs into the draw call
        text.compose_draw_call(&mut renderer);

        // Render the OpenGL draw calls
        renderer.render(window.width, window.height);

        // Swap the buffers and wait for the window to refresh (this
        // is referring to vsync, and will usually take ~16ms at max)
        window.swap_buffers(Some(&renderer));
    }
    Ok(())
}

#[cfg(feature = "glfw")]
mod keys {
    pub const CLOSE: glfw::Key = glfw::Key::Escape;
}

#[cfg(feature = "glutin")]
mod keys {
    pub const CLOSE: glutin::VirtualKeyCode = glutin::VirtualKeyCode::Escape;
}

#[cfg(not(any(feature = "glutin", feature = "glfw")))]
mod keys {
    pub const CLOSE: u32 = 27;
}
