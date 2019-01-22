use crate::renderer::Renderer;
use std::error::Error;

pub use crate::window_settings::WindowSettings;

/// Manages the window and propagates events to the UI system.
pub struct Window {
    /// The width of the window.
    pub width: f32,
    /// The height of the window.
    pub height: f32,
    /// The dpi of the window.
    pub dpi_factor: f32,
    /// The opengl legacy status for Renderer.
    pub opengl21: bool,
    /// The keys which are currently held down. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub pressed_keys: Vec<u32>,
    /// The keys which were pressed this frame. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub just_pressed_keys: Vec<u32>,
    /// The keys which were released this frame. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub released_keys: Vec<u32>,
}

impl Window {
    /// Creates a new `Window`.
    ///
    /// Can result in an error if window creation fails or OpenGL
    /// context creation fails.
    #[allow(unused_variables)]
    pub fn create(settings: &WindowSettings) -> Result<Window, Box<Error>> {
        Err(Box::new(WindowCreationError))
    }

    /// Updates the window (swaps the front and back buffers)
    pub fn swap_buffers(&mut self, renderer: &Renderer) {}

    /// Polls for new events. Returns whether the user has requested
    /// the window to be closed. **Note**: Because of vsync, this
    /// function will hang for a while (usually 16ms at max).
    pub fn refresh(&mut self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
struct WindowCreationError;

impl std::fmt::Display for WindowCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "cannot create window without glutin or glfw")
    }
}

impl Error for WindowCreationError {
    fn description(&self) -> &str {
        "cannot create window without glutin or glfw"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
