//! This is mostly just explaining what's happening behind the scenes (in
//! case you need to know to debug something), if you use the `window`
//! mod, your programs should be automatically HiDPI aware and work
//! without any additional work. Just remember that the pixels you're
//! working in while using this crate are *logical pixels*, not physical
//! pixels. That means that a 640x480 window is actually 1280x960 physical
//! pixels if you're running on a Retina/HiDPI monitor with 2x scaling,
//! and all your rendering is scaled accordingly.
//!
//! On Windows and macOS, window scaling is done as you'd expect out of a
//! HiDPI aware program. On Linux, it's different for x11 and
//! wayland. Here's how glutin and glfw handle the situations:
//! - Glutin + x11: Glutin will scale the window according to the screen's
//!   actual DPI, resulting in pretty good results across low- and
//!   high-dpi screens.
//! - Glfw + x11: Glfw does not scale anything, but fae manually applies
//!   the multiplier mentioned below.
//! - Glutin + wayland: Glutin will scale the window according to the
//!   scale factor reported by the Wayland environment. But because
//!   Xwayland windows look bad when scaled (they're just stretched), my
//!   Wayland setup has the scale set to 1. Because of this, fae applies
//!   the multiplier below as well. So if your Wayland environment has
//!   scale set to 2, and your GDK_SCALE is 2, your fae application will
//!   render at 4x. This behavior seems the most consistent with other
//!   applications I happen to use, if you have other suggestions, I'm
//!   open to discussion.
//! - Glfw + wayland: Glfw will run in Xwayland, and so it'll very
//!   probably be scaled by your WM as well as the environment variables
//!   below. This should be consistent with Glutin+wayland behavior,
//!   except that the wayland scaling factor is applied by the WM, so the
//!   result will be blurry for scaling factors greater than 1.
//!
//! Environment variables that will be considered multipliers for the dpi
//! factor on Glfw and Glutin+wayland (the first non-0 is used):
//! - `QT_AUTO_SCREEN_SCALE_FACTOR`
//! - `QT_SCALE_FACTOR`
//! - `GDK_SCALE`
//! - `ELM_SCALE`

use crate::mouse::Mouse;
use crate::renderer::Renderer;
use std::error::Error;
use std::path::PathBuf;

pub use crate::window_util::WindowSettings;

/// Wrapper for a Glutin/Glfw window.
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
    pub held_keys: Vec<u32>,
    /// The keys which were pressed this frame. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub pressed_keys: Vec<u32>,
    /// The keys which were released this frame. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub released_keys: Vec<u32>,
    /// The characters typed this frame, in chronological order.
    pub typed_chars: Vec<char>,

    /// Whether the mouse is inside the window.
    pub mouse_inside: bool,
    /// The mouse position inside the window. Arrangement: (x, y)
    pub mouse_coords: (f32, f32),
    /// The mouse scroll amount during this frame, in pixels. If the
    /// user supports pixel-perfect scrolling, this will be equal to
    /// those pixel-perfect deltas. Otherwise, the polled scrolling
    /// amounts will be multiplied with `mouse_scroll_length`. With
    /// the default settings, this will usually result in bursts of
    /// (0, -36) and (0, 36) during normal scrolling. Arrangement: (x,
    /// y)
    pub mouse_scroll: (f32, f32),
    /// How many pixels one "notch" on the mouse scroll wheel should
    /// scroll. (36 by default)
    pub mouse_scroll_length: f32,
    /// The mouse buttons which are currently held down.
    pub mouse_held: Vec<Mouse>,
    /// The mouse buttons which were pressed down this frame.
    pub mouse_pressed: Vec<Mouse>,
    /// The mouse buttons which were released this frame.
    pub mouse_released: Vec<Mouse>,

    /// A list of files dropped on the window during this frame.
    pub dropped_files: Vec<PathBuf>,
    /// A list of files being currently hovered on the window. Does
    /// not work if using the GLFW backend.
    pub hovered_files: Vec<PathBuf>,
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

    /// Sets the cursor graphic to the provided one. NOTE: This
    /// function has a different signature in Glutin and Glfw, so take
    /// that into account when using this. `cursor`'s type is
    /// `glutin::MouseCursor` or `glfw::StandardCursor` depending on
    /// your features.
    #[allow(unused_variables)]
    pub fn set_cursor(&mut self, cursor: u32) {}

    /// Updates the window (swaps the front and back buffers). The
    /// renderer handle is used for a CPU/GPU synchronization call, so
    /// while it is optional, it's definitely recommended. If vsync is
    /// enabled, this function will hang until the next frame.
    #[allow(unused_variables)]
    pub fn swap_buffers(&mut self, renderer: Option<&Renderer>) {}

    /// Polls for new events. Returns whether the user has requested
    /// the window to be closed.
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
