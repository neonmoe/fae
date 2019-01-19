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

    /// Re-renders the window, polls for new events and passes them on
    /// to the UI system, and clears the screen with the
    /// `background_*` colors, which consist of 0.0 - 1.0
    /// values. **Note**: Because of vsync, this function will hang
    /// for a while (usually 16ms at max).
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
