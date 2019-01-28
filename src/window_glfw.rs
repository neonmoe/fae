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

use crate::gl;
use crate::mouse::Mouse;
use crate::renderer::Renderer;
use glfw::*;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

pub use crate::window_util::*;
pub use glfw;

const HIDPI_AUTO: bool = cfg!(any(target_os = "windows", target_os = "macos"));

/// Wrapper for a Glutin/Glfw window.
pub struct Window {
    /// The width of the window.
    pub width: f32,
    /// The height of the window.
    pub height: f32,
    /// The dpi of the window.
    pub dpi_factor: f32,
    glfw: glfw::Glfw,
    glfw_window: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    fb_width: f32,
    fb_height: f32,
    /// The opengl legacy status for Renderer.
    pub opengl21: bool,
    /// The keys which are currently held down. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub held_keys: Vec<Key>,
    /// The keys which were pressed this frame. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub pressed_keys: Vec<Key>,
    /// The keys which were released this frame. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub released_keys: Vec<Key>,
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
    pub fn create(settings: &WindowSettings) -> Result<Window, Box<Error>> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

        let mut width = settings.width;
        let mut height = settings.height;
        let dpi_factor;
        if !HIDPI_AUTO {
            dpi_factor = get_env_dpi();
            width /= dpi_factor;
            height /= dpi_factor;
        } else {
            dpi_factor = 1.0;
        }

        let opengl21 = false;
        let (mut glfw_window, events) = {
            // Forward compatibility flag needed for mac:
            // https://www.khronos.org/opengl/wiki/OpenGL_Context#Forward_compatibility
            if cfg!(target_os = "macos") {
                glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
            }
            glfw.window_hint(WindowHint::SRgbCapable(true));

            let width = width as u32;
            let height = height as u32;
            let title = &settings.title;
            let window_mode = glfw::WindowMode::Windowed;

            if env::var_os("FAE_OPENGL_LEGACY").is_some() {
                if let Some(result) = {
                    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::OpenGl));
                    glfw.window_hint(WindowHint::ContextVersion(2, 1));
                    glfw.create_window(width, height, title, window_mode)
                } {
                    result
                } else if let Some(result) = {
                    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::OpenGlEs));
                    glfw.window_hint(WindowHint::ContextVersion(2, 0));
                    glfw.create_window(width, height, title, window_mode)
                } {
                    result
                } else {
                    return Err(Box::new(WindowCreationError));
                }
            } else {
                if let Some(result) = {
                    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::OpenGl));
                    glfw.window_hint(WindowHint::ContextVersion(3, 3));
                    glfw.create_window(width, height, title, window_mode)
                } {
                    result
                } else if let Some(result) = {
                    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::OpenGlEs));
                    glfw.window_hint(WindowHint::ContextVersion(3, 0));
                    glfw.create_window(width, height, title, window_mode)
                } {
                    result
                } else if let Some(result) = {
                    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::OpenGl));
                    glfw.window_hint(WindowHint::ContextVersion(2, 1));
                    glfw.create_window(width, height, title, window_mode)
                } {
                    result
                } else if let Some(result) = {
                    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::OpenGlEs));
                    glfw.window_hint(WindowHint::ContextVersion(2, 0));
                    glfw.create_window(width, height, title, window_mode)
                } {
                    result
                } else {
                    return Err(Box::new(WindowCreationError));
                }
            }
        };

        glfw_window.make_current();
        gl::load_with(|symbol| glfw_window.get_proc_address(symbol) as *const _);
        /* use std::ffi::CStr;

            Uncomment in case of opengl shenanigans

            let ptr = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_bytes();
            let opengl_version_string = String::from_utf8_lossy(ptr);
            if cfg!(debug_assertions) {
            println!("OpenGL version: {}", opengl_version_string);
        }*/

        if settings.vsync {
            if glfw.extension_supported("WGL_EXT_swap_control_tear")
                || glfw.extension_supported("GLX_EXT_swap_control_tear")
            {
                glfw.set_swap_interval(glfw::SwapInterval::Adaptive);
            } else {
                glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
            }
        } else {
            glfw.set_swap_interval(glfw::SwapInterval::None);
        }

        glfw_window.set_all_polling(true);

        Ok(Window {
            width: width,
            height: height,
            dpi_factor,
            glfw,
            glfw_window,
            events,
            fb_width: width,
            fb_height: height,
            opengl21,

            held_keys: Vec::new(),
            pressed_keys: Vec::new(),
            released_keys: Vec::new(),
            typed_chars: Vec::new(),

            mouse_inside: false,
            mouse_coords: (0.0, 0.0),
            mouse_scroll: (0.0, 0.0),
            mouse_scroll_length: 36.0,
            mouse_held: Vec::new(),
            mouse_pressed: Vec::new(),
            mouse_released: Vec::new(),

            dropped_files: Vec::new(),
            hovered_files: Vec::new(),
        })
    }

    /// Sets the cursor graphic to the provided one. NOTE: This
    /// function has a different signature in Glutin and Glfw, so take
    /// that into account when using this.
    pub fn set_cursor(&mut self, cursor: StandardCursor) {
        self.glfw_window.set_cursor(Some(Cursor::standard(cursor)));
    }

    /// Updates the window (swaps the front and back buffers). The
    /// renderer handle is used for a CPU/GPU synchronization call, so
    /// while it is optional, it's definitely recommended. If vsync is
    /// enabled, this function will hang until the next frame.
    pub fn swap_buffers(&mut self, renderer: Option<&Renderer>) {
        self.glfw_window.swap_buffers();
        if let Some(renderer) = renderer {
            renderer.synchronize();
        }
    }

    /// Polls for new events. Returns whether the user has requested
    /// the window to be closed.
    pub fn refresh(&mut self) -> bool {
        let mut resize = false;

        self.pressed_keys.clear();
        self.released_keys.clear();
        self.typed_chars.clear();
        self.mouse_scroll = (0.0, 0.0);
        self.dropped_files.clear();

        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Key(key, _, Action::Press, _) => {
                    self.pressed_keys.push(key);
                    self.held_keys.push(key);
                }
                WindowEvent::Key(key, _, Action::Release, _) => {
                    self.released_keys.push(key);
                    for (i, held_key) in self.held_keys.iter().enumerate() {
                        if held_key == &key {
                            self.held_keys.remove(i);
                            break;
                        }
                    }
                }

                WindowEvent::Char(c) => self.typed_chars.push(c),

                WindowEvent::MouseButton(button, action, _) => {
                    let button = match button {
                        MouseButton::Button1 => Mouse::Left,
                        MouseButton::Button2 => Mouse::Right,
                        MouseButton::Button3 => Mouse::Middle,
                        MouseButton::Button4 => Mouse::Other(4),
                        MouseButton::Button5 => Mouse::Other(5),
                        MouseButton::Button6 => Mouse::Other(6),
                        MouseButton::Button7 => Mouse::Other(7),
                        MouseButton::Button8 => Mouse::Other(8),
                    };

                    match action {
                        Action::Press => {
                            self.mouse_pressed.push(button);
                            self.mouse_held.push(button);
                        }
                        Action::Release => {
                            self.mouse_released.push(button);
                            for (i, held_button) in self.mouse_held.iter().enumerate() {
                                if held_button == &button {
                                    self.mouse_held.remove(i);
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                WindowEvent::CursorPos(x, y) => {
                    self.mouse_coords = (x as f32 / self.dpi_factor, y as f32 / self.dpi_factor);
                }
                WindowEvent::CursorEnter(entered) => self.mouse_inside = entered,

                WindowEvent::Scroll(x, y) => {
                    self.mouse_scroll = (
                        self.mouse_scroll_length * x as f32,
                        self.mouse_scroll_length * y as f32,
                    )
                }

                WindowEvent::FileDrop(paths) => self.dropped_files = paths,

                WindowEvent::Size(width, height) => {
                    if HIDPI_AUTO {
                        self.width = width as f32;
                        self.height = height as f32;
                    } else {
                        self.width = width as f32 / self.dpi_factor;
                        self.height = height as f32 / self.dpi_factor;
                    }
                    resize = true;
                }
                WindowEvent::FramebufferSize(width, height) => {
                    self.fb_width = width as f32;
                    self.fb_height = height as f32;
                    resize = true;
                }

                _ => {}
            }
        }

        if resize {
            unsafe {
                gl::Viewport(0, 0, self.fb_width as i32, self.fb_height as i32);
            }

            // GLFW framebuffer and screen space sizes only differ on windows and macos
            if HIDPI_AUTO {
                let dpi_factor_horizontal = self.fb_width / self.width;
                let dpi_factor_vertical = self.fb_height / self.height;
                self.dpi_factor = dpi_factor_horizontal.max(dpi_factor_vertical);
            }
        }

        !self.glfw_window.should_close()
    }
}

#[derive(Debug, Clone)]
struct WindowCreationError;

impl std::fmt::Display for WindowCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "could not create a glfw window")
    }
}

impl Error for WindowCreationError {
    fn description(&self) -> &str {
        "could not create a glfw window"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
