use crate::gl;
use glutin::dpi::*;
use glutin::*;
use std::default::Default;
use std::env;
use std::error::Error;

pub use glutin;

/// Defines a window.
pub struct WindowSettings {
    /// Title of the window. Default value: Name of the executable file
    pub title: String,
    /// Width of the window in logical pixels. Default value: `640.0`
    pub width: f32,
    /// Height of the window in logical pixels. Default value: `480.0`
    pub height: f32,
    /// Whether or not the application is a dialog. Default value: `true`
    ///
    /// This only affects x11 environments, where it sets the window
    /// type to dialog. In [tiling
    /// environments](https://en.wikipedia.org/wiki/Tiling_window_manager),
    /// like i3 and sway, this can cause the window to pop up as a
    /// floating window, not a tiled one. This is useful for
    /// applications that are supposed to be opened for very short
    /// amounts of time.
    pub is_dialog: bool,
    /// This should always be true for everything except benchmarks.
    pub vsync: bool,
}

impl Default for WindowSettings {
    fn default() -> WindowSettings {
        WindowSettings {
            title: env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_os_string()))
                .and_then(|s| s.into_string().ok())
                .unwrap_or_default(),
            width: 640.0,
            height: 480.0,
            is_dialog: false,
            vsync: true,
        }
    }
}

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
}

impl Window {
    /// Creates a new `Window`.
    ///
    /// Can result in an error if window creation fails or OpenGL
    /// context creation fails.
    pub fn create(settings: WindowSettings) -> Result<Window, Box<Error>> {
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
        })
    }

    /// Re-renders the window, polls for new events and passes them on
    /// to the UI system, and clears the screen with the
    /// `background_*` colors, which consist of 0.0 - 1.0
    /// values. **Note**: Because of vsync, this function will hang
    /// for a while (usually 16ms at max).
    pub fn refresh<F: FnMut(&Event)>(&mut self, mut event_handler: F) -> bool {
        let _ = self.gl_window.swap_buffers();
        let mut running = true;
        let mut resized_logical_size = None;
        self.events_loop.poll_events(|event| {
            event_handler(&event);
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => running = false,
                    WindowEvent::Resized(logical_size) => resized_logical_size = Some(logical_size),
                    _ => {}
                }
            }
        });

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
