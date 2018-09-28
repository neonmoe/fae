mod frame_timer;

use self::frame_timer::FrameTimer;
use gl;
use glutin::dpi::*;
use glutin::*;
use renderer;
#[cfg(feature = "default_resources")]
use resources;
use std::default::Default;
use std::env;
use std::error::Error;
use std::time::Duration;
use text;
use ui::{self, MouseStatus};

/// Defines a window.
pub struct WindowSettings {
    /// Title of the window. Default value: Name of the executable file
    pub title: String,
    /// Width of the window in logical pixels. Default value: `320.0`
    pub width: f64,
    /// Height of the window in logical pixels. Default value: `240.0`
    pub height: f64,
    /// Whether or not the application is a dialog. Default value: `true`
    ///
    /// This only affects x11 environments, where it sets the window
    /// type to dialog. In [tiling
    /// environments](https://en.wikipedia.org/wiki/Tiling_window_manager),
    /// like i3 and sway, this can cause the window to pop up as a
    /// floating window, not a tiled one. This is useful for
    /// applications that have very fixed layouts, and are supposed to
    /// be opened for very short amounts of time at a time, like small
    /// utilities.
    pub is_dialog: bool,
    /// The bytes of a .png file with an alpha channel, which will be
    /// used as the application's spritesheet. Default value:
    /// `resources::DEFAULT_UI_SPRITESHEET.to_vec()`
    ///
    /// ![Default GUI][gui]
    ///
    /// The layout of the gui spritesheet is very important. Say `H`
    /// is the height of the texture. When the rendering engine is
    /// looking for a specific element's sprite, it will look for
    /// `HxH` areas in the texture, and consider these areas to
    /// consist of a uniform 3x3 grid. The center tile will be
    /// stretched to the width and height of a given element, the top
    /// and bottom tiles will stretch to the width of the element, and
    /// the left and right tiles will stretch to the height of the
    /// element. The corner tiles will only be stretched uniformly, as
    /// described in the following section.
    ///
    /// The resolution of the texture is irrelevant, as all tiles will
    /// be stretched to be 16x16 logical pixels, except for the
    /// width/height stretching cases described in the previous
    /// section. To not lose pixel density in HiDPI situations, make
    /// your tiles 32x32 or higher. At 32x32 tiles, your spritesheet's
    /// height would be 96px.
    ///
    /// The elements should all be on one row, and right next to each
    /// other. For a reference, cut up the default
    /// [`gui.png`][gui] into 16x16 chunks, and modify
    /// from there.
    ///
    /// [gui]: https://git.neon.moe/neon/fungui/raw/branch/master/src/resources/gui.png
    pub ui_spritesheet: Vec<u8>,
    /// The bytes of a .ttf file that will be used as the
    /// application's font. Default value: `resources::DEFAULT_FONT.to_vec()`
    ///
    /// The default font provided by the `default_resources` feature
    /// is Fira Sans.
    pub font: Vec<u8>,
}

impl Default for WindowSettings {
    fn default() -> WindowSettings {
        WindowSettings {
            title: env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_os_string()))
                .and_then(|s| s.into_string().ok())
                .unwrap_or_default(),
            width: 320.0,
            height: 240.0,
            is_dialog: true,
            ui_spritesheet: get_default_ui_spritesheet(),
            font: get_default_font(),
        }
    }
}

#[cfg(feature = "default_resources")]
fn get_default_ui_spritesheet() -> Vec<u8> {
    resources::DEFAULT_UI_SPRITESHEET.to_vec()
}
#[cfg(not(feature = "default_resources"))]
fn get_default_ui_spritesheet() -> Vec<u8> {
    panic!("default_resources feature is disabled, but no UI spritesheet was provided!");
}

#[cfg(feature = "default_resources")]
fn get_default_font() -> Vec<u8> {
    resources::DEFAULT_FONT.to_vec()
}
#[cfg(not(feature = "default_resources"))]
fn get_default_font() -> Vec<u8> {
    panic!("default_resources feature is disabled, but no font was provided!");
}

/// Manages the window and propagates events to the UI system.
pub struct Window {
    width: f64,
    height: f64,
    dpi: f64,
    gl_window: GlWindow,
    events_loop: EventsLoop,
    mouse: MouseStatus,
    frame_timer: FrameTimer,
}

impl Window {
    /// Creates a new `Window`.
    ///
    /// Can result in an error if window creation fails, OpenGL
    /// context creation fails, or resources defined in the `settings`
    /// can't be loaded.
    pub fn new(settings: WindowSettings) -> Result<Window, Box<Error>> {
        // FIXME: Enable wayland support by not setting the backend to
        // x11 automatically. Note: At the time of writing, wayland
        // support in winit seems to be buggy. At the very least, it
        // doesn't seem to work with the sway the
        env::set_var("WINIT_UNIX_BACKEND", "x11");

        let events_loop = EventsLoop::new();
        let mut window = WindowBuilder::new()
            .with_title(settings.title)
            .with_dimensions(LogicalSize::new(settings.width, settings.height));
        if settings.is_dialog {
            window = Window::window_as_dialog(window);
        }
        let context = ContextBuilder::new()
            .with_vsync(true)
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core);
        let gl_window = GlWindow::new(window, context, &events_loop)?;

        unsafe {
            gl_window.make_current()?;
            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.85, 0.85, 0.85, 1.0);
        }

        renderer::initialize_renderer(&settings.ui_spritesheet)?;
        text::initialize_font(settings.font)?;

        Ok(Window {
            width: settings.width,
            height: settings.height,
            dpi: 1.0,
            gl_window,
            events_loop,
            mouse: MouseStatus {
                x: 0.0,
                y: 0.0,
                last_pressed: false,
                pressed: false,
            },
            frame_timer: FrameTimer::new(),
        })
    }

    /// Re-renders the window, polls for new events, and passes them
    /// on to the UI system. **Note**: Because of vsync, this function
    /// will hang for a while (usually 16ms at max).
    pub fn refresh(&mut self) -> bool {
        let mut running = true;

        self.frame_timer.end_frame();
        let _ = self.gl_window.swap_buffers();
        self.frame_timer.begin_frame();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let mut resized_logical_size = None;
        let mut mouse_position = None;
        let mut mouse_pressed = None;
        self.events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => running = false,
                    WindowEvent::Resized(logical_size) => resized_logical_size = Some(logical_size),
                    WindowEvent::CursorMoved { position, .. } => mouse_position = Some(position),
                    WindowEvent::MouseInput { state, .. } => {
                        mouse_pressed = Some(state == ElementState::Pressed)
                    }
                    _ => (),
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
            self.width = logical_size.width;
            self.height = logical_size.height;
            self.dpi = dpi_factor;
        }

        /* Mouse move event handling */
        if let Some(position) = mouse_position {
            self.mouse.x = position.x as f32;
            self.mouse.y = position.y as f32;
        }

        /* Mouse button event handling */
        self.mouse.last_pressed = self.mouse.pressed;
        if let Some(pressed) = mouse_pressed {
            self.mouse.pressed = pressed;
        }

        ui::update(self.width, self.height, self.dpi as f32, self.mouse);

        running
    }

    /// Returns the average duration of the last 60 frames. A "frame"
    /// includes operations between the latest refresh() and the one
    /// before that, except waiting for vsync.
    pub fn avg_frame_duration(&self) -> Duration {
        self.frame_timer.avg_frame_duration()
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
