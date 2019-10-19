use crate::gl;
use crate::renderer::Renderer;
pub use crate::window::WindowSettings;
use crate::window::{get_env_dpi, Mouse};
pub use glutin;
use glutin::dpi::*;
use glutin::*;
use std::env;
use std::error::Error;
use std::path::PathBuf;

/// Wrapper for a Glutin window.
pub struct Window {
    env_dpi_factor: f32,
    context: WindowedContext<PossiblyCurrent>,
    events_loop: EventsLoop,

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
    pub held_keys: Vec<VirtualKeyCode>,
    /// The keys which were pressed this frame. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub pressed_keys: Vec<VirtualKeyCode>,
    /// The keys which were released this frame. Different type for
    /// each window backend, because there's no unified way of
    /// speaking in keycodes!
    pub released_keys: Vec<VirtualKeyCode>,
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
    /// A list of files being currently hovered on the window.
    pub hovered_files: Vec<PathBuf>,
}

impl Window {
    /// Creates a new `Window`.
    ///
    /// Can result in an error if window creation fails or OpenGL
    /// context creation fails.
    pub fn create(settings: &WindowSettings) -> Result<Window, Box<dyn Error>> {
        let events_loop = EventsLoop::new();
        let opengl21;
        let context = {
            let create_window = |gl_request, gl_profile| {
                let mut window = WindowBuilder::new()
                    .with_title(settings.title.clone())
                    .with_dimensions(LogicalSize::new(
                        f64::from(settings.width),
                        f64::from(settings.height),
                    ))
                    .with_visibility(false);
                if settings.is_dialog {
                    window = window_as_dialog(window);
                }
                ContextBuilder::new()
                    .with_vsync(settings.vsync)
                    .with_srgb(true)
                    .with_gl(gl_request)
                    .with_gl_profile(gl_profile)
                    .build_windowed(window, &events_loop)
            };

            let gl_req_21 = GlRequest::GlThenGles {
                opengl_version: (2, 1),
                opengles_version: (2, 0),
            };
            let gl_req_33 = GlRequest::GlThenGles {
                opengl_version: (3, 3),
                opengles_version: (3, 0),
            };
            if env::var_os("FAE_OPENGL_LEGACY").is_some() {
                opengl21 = true;
                create_window(gl_req_21, GlProfile::Compatibility)?
            } else if let Ok(result) = create_window(gl_req_33, GlProfile::Core) {
                opengl21 = false;
                result
            } else {
                opengl21 = true;
                create_window(gl_req_21, GlProfile::Compatibility)?
            }
        };

        let env_dpi_factor = {
            let multiplier = get_env_dpi();
            if let Some(size) = context.window().get_inner_size() {
                let (w, h): (f64, f64) = size.into();
                context
                    .window()
                    .set_inner_size((w * multiplier as f64, h * multiplier as f64).into());
            }
            multiplier
        };

        let context = unsafe {
            let context = match context.make_current() {
                Ok(current_ctx) => current_ctx,
                Err((_, err)) => return Err(Box::new(err)),
            };
            gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
            /* use std::ffi::CStr;

            Uncomment in case of opengl shenanigans

            let opengl_version_string = String::from_utf8_lossy(
                CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_bytes(),
            );
            if cfg!(debug_assertions) {
                println!("OpenGL version: {}", opengl_version_string);
            }*/
            context
        };

        context.window().show();

        Ok(Window {
            env_dpi_factor,
            context,
            events_loop,

            width: settings.width,
            height: settings.height,
            dpi_factor: 1.0,
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

    /// Sets the cursor graphic to the provided one.
    pub fn set_cursor(&self, cursor: MouseCursor) {
        self.context.window().set_cursor(cursor);
    }

    /// Updates the window (swaps the front and back buffers). The
    /// renderer handle is used for a CPU/GPU synchronization call, so
    /// while it is optional, it's definitely recommended. If vsync is
    /// enabled, this function will hang until the next frame.
    pub fn swap_buffers(&mut self, renderer: Option<&Renderer>) {
        let _ = self.context.swap_buffers();
        if let Some(renderer) = renderer {
            renderer.synchronize();
        }
    }

    /// Polls for new events. Returns whether the user has requested
    /// the window to be closed.
    pub fn refresh(&mut self) -> bool {
        let mut running = true;
        let mut key_inputs = Vec::new();
        let mut mouse_inputs = Vec::new();
        let window_width = &mut self.width;
        let window_height = &mut self.height;
        let window_dpi_factor = &mut self.dpi_factor;
        let typed_chars = &mut self.typed_chars;
        let mouse_coords = &mut self.mouse_coords;
        let mouse_inside = &mut self.mouse_inside;
        let scroll = &mut self.mouse_scroll;
        let scroll_length = self.mouse_scroll_length;
        let dropped_files = &mut self.dropped_files;
        let hovered_files = &mut self.hovered_files;

        *scroll = (0.0, 0.0);
        typed_chars.clear();
        dropped_files.clear();

        let context = &self.context;
        let env_dpi_factor = self.env_dpi_factor;
        let mut handle_resize = |logical_size: LogicalSize, dpi_factor: f64| {
            let physical_size = logical_size.to_physical(dpi_factor);

            let (width, height): (u32, u32) = physical_size.into();
            unsafe {
                gl::Viewport(0, 0, width as i32, height as i32);
            }
            context.resize(physical_size);
            *window_width = logical_size.width as f32 / env_dpi_factor;
            *window_height = logical_size.height as f32 / env_dpi_factor;
            *window_dpi_factor = dpi_factor as f32 * env_dpi_factor;
        };

        let events_loop = &mut self.events_loop;
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => running = false,
                    WindowEvent::Resized(logical_size) => {
                        let dpi_factor = context.window().get_hidpi_factor();
                        handle_resize(logical_size, dpi_factor);
                    }
                    WindowEvent::HiDpiFactorChanged(dpi_factor) => {
                        if let Some(logical_size) = context.window().get_inner_size() {
                            handle_resize(logical_size, dpi_factor);
                        }
                    }

                    WindowEvent::KeyboardInput { input, .. } => {
                        let state = input.state;
                        if let Some(key) = input.virtual_keycode {
                            key_inputs.push((key, state));
                        }
                    }
                    WindowEvent::ReceivedCharacter(c) => typed_chars.push(c),

                    WindowEvent::MouseInput { state, button, .. } => match button {
                        MouseButton::Left => mouse_inputs.push((Mouse::Left, state)),
                        MouseButton::Right => mouse_inputs.push((Mouse::Right, state)),
                        MouseButton::Middle => mouse_inputs.push((Mouse::Middle, state)),
                        MouseButton::Other(n) => mouse_inputs.push((Mouse::Other(n + 3), state)),
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        *mouse_coords = (position.x as f32, position.y as f32);
                    }
                    WindowEvent::CursorEntered { .. } => *mouse_inside = true,
                    WindowEvent::CursorLeft { .. } => *mouse_inside = false,
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        MouseScrollDelta::LineDelta(x, y) => {
                            *scroll = (scroll_length * x, scroll_length * y)
                        }
                        MouseScrollDelta::PixelDelta(pos) => *scroll = (pos.x as f32, pos.y as f32),
                    },

                    WindowEvent::DroppedFile(path) => {
                        for (i, hovered_path) in hovered_files.iter().enumerate() {
                            if hovered_path == &path {
                                hovered_files.remove(i);
                                break;
                            }
                        }
                        dropped_files.push(path);
                    }
                    WindowEvent::HoveredFile(path) => {
                        hovered_files.push(path);
                    }
                    WindowEvent::HoveredFileCancelled => {
                        hovered_files.clear();
                    }

                    _ => {}
                }
            }
        });

        /* Keyboard event handling */
        self.pressed_keys.clear();
        self.released_keys.clear();
        for (key, state) in key_inputs {
            match state {
                ElementState::Pressed => {
                    let mut already_pressed = false;
                    for previously_pressed_key in &self.held_keys {
                        if previously_pressed_key == &key {
                            already_pressed = true;
                            break;
                        }
                    }

                    if !already_pressed {
                        self.pressed_keys.push(key);
                        self.held_keys.push(key);
                    }
                }
                ElementState::Released => {
                    self.released_keys.push(key);
                    for (i, held_key) in self.held_keys.iter().enumerate() {
                        if held_key == &key {
                            self.held_keys.remove(i);
                            break;
                        }
                    }
                }
            }
        }

        /* Mouse event handling */
        self.mouse_pressed.clear();
        self.mouse_released.clear();
        for (button, state) in mouse_inputs {
            match state {
                ElementState::Pressed => {
                    self.mouse_pressed.push(button);
                    self.mouse_held.push(button);
                }
                ElementState::Released => {
                    self.mouse_released.push(button);
                    for (i, held_button) in self.mouse_held.iter().enumerate() {
                        if held_button == &button {
                            self.mouse_held.remove(i);
                            break;
                        }
                    }
                }
            }
        }

        running
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd"
))]
fn window_as_dialog(window: WindowBuilder) -> WindowBuilder {
    use glutin::os::unix::{WindowBuilderExt, XWindowType};
    window.with_x11_window_type(XWindowType::Dialog)
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd"
)))]
fn window_as_dialog(window: WindowBuilder) -> WindowBuilder {
    window
}
