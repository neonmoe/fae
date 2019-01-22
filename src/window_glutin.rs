use crate::gl;
use crate::mouse::Mouse;
use crate::renderer::Renderer;
use glutin::dpi::*;
use glutin::*;
use std::env;
use std::error::Error;

pub use crate::window_settings::WindowSettings;
pub use glutin;

/// Manages the window and propagates events to the UI system.
pub struct Window {
    /// The width of the window.
    pub width: f32,
    /// The height of the window.
    pub height: f32,
    /// The dpi of the window.
    pub dpi_factor: f32,
    gl_window: GlWindow,
    events_loop: EventsLoop,
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
    /// The mouse buttons which are currently held down.
    pub mouse_held: Vec<Mouse>,
    /// The mouse buttons which were pressed down this frame.
    pub mouse_pressed: Vec<Mouse>,
    /// The mouse buttons which were released this frame.
    pub mouse_released: Vec<Mouse>,
}

impl Window {
    /// Creates a new `Window`.
    ///
    /// Can result in an error if window creation fails or OpenGL
    /// context creation fails.
    pub fn create(settings: &WindowSettings) -> Result<Window, Box<Error>> {
        // Note: At the time of writing, wayland support in winit
        // seems to be buggy. Default to x11, since xwayland at least
        // works.
        if cfg!(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "openbsd",
        )) {
            env::set_var("WINIT_UNIX_BACKEND", "x11");
        }

        let events_loop = EventsLoop::new();
        let opengl21;
        let gl_window = {
            let create_window = |gl_request, gl_profile| {
                let mut window = WindowBuilder::new()
                    .with_title(settings.title.clone())
                    .with_dimensions(LogicalSize::new(
                        f64::from(settings.width),
                        f64::from(settings.height),
                    ));
                if settings.is_dialog {
                    window = Window::window_as_dialog(window);
                }
                let context = ContextBuilder::new()
                    .with_vsync(settings.vsync)
                    .with_srgb(true)
                    .with_gl(gl_request)
                    .with_gl_profile(gl_profile);
                GlWindow::new(window, context, &events_loop)
            };

            if env::var_os("FAE_OPENGL_LEGACY").is_some() {
                opengl21 = true;
                create_window(
                    GlRequest::GlThenGles {
                        opengl_version: (2, 1),
                        opengles_version: (2, 0),
                    },
                    GlProfile::Compatibility,
                )?
            } else if let Ok(result) = create_window(
                GlRequest::GlThenGles {
                    opengl_version: (3, 3),
                    opengles_version: (3, 0),
                },
                GlProfile::Core,
            ) {
                opengl21 = false;
                result
            } else {
                opengl21 = true;
                create_window(
                    GlRequest::GlThenGles {
                        opengl_version: (2, 1),
                        opengles_version: (2, 0),
                    },
                    GlProfile::Compatibility,
                )?
            }
        };

        unsafe {
            gl_window.make_current()?;
            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
            /* use std::ffi::CStr;

            Uncomment in case of opengl shenanigans

            let opengl_version_string = String::from_utf8_lossy(
                CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_bytes(),
            );
            if cfg!(debug_assertions) {
                println!("OpenGL version: {}", opengl_version_string);
            }*/
        }

        Ok(Window {
            width: settings.width,
            height: settings.height,
            dpi_factor: 1.0,
            gl_window,
            events_loop,
            opengl21,

            held_keys: Vec::new(),
            pressed_keys: Vec::new(),
            released_keys: Vec::new(),
            typed_chars: Vec::new(),

            mouse_inside: false,
            mouse_coords: (0.0, 0.0),
            mouse_held: Vec::new(),
            mouse_pressed: Vec::new(),
            mouse_released: Vec::new(),
        })
    }

    /// Updates the window (swaps the front and back buffers)
    pub fn swap_buffers(&mut self, renderer: &Renderer) {
        let _ = self.gl_window.swap_buffers();
        renderer.synchronize();
    }

    /// Polls for new events. Returns whether the user has requested
    /// the window to be closed. **Note**: Because of vsync, this
    /// function will hang for a while (usually 16ms at max).
    pub fn refresh(&mut self) -> bool {
        let mut running = true;
        let mut resized_logical_size = None;
        let mut key_inputs = Vec::new();
        let mut mouse_inputs = Vec::new();
        let typed_chars = &mut self.typed_chars;
        let mouse_coords = &mut self.mouse_coords;
        let mouse_inside = &mut self.mouse_inside;
        typed_chars.clear();

        self.events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => running = false,
                    WindowEvent::Resized(logical_size) => resized_logical_size = Some(logical_size),
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

        /* Resize event handling */
        if let Some(logical_size) = resized_logical_size {
            let dpi_factor = self.gl_window.get_hidpi_factor();
            let physical_size = logical_size.to_physical(dpi_factor);

            let (width, height): (u32, u32) = physical_size.into();
            unsafe {
                gl::Viewport(0, 0, width as i32, height as i32);
            }
            self.gl_window.resize(physical_size);
            self.width = logical_size.width as f32;
            self.height = logical_size.height as f32;
            self.dpi_factor = dpi_factor as f32;
        }

        running
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
}
