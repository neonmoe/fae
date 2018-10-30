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
use ui::{self, KeyStatus, MouseStatus};

/// Defines a window.
pub struct WindowSettings {
    /// Title of the window. Default value: Name of the executable file
    pub title: String,
    /// Width of the window in logical pixels. Default value: `320.0`
    pub width: f32,
    /// Height of the window in logical pixels. Default value: `240.0`
    pub height: f32,
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
    Vec::new()
}

#[cfg(feature = "default_resources")]
fn get_default_font() -> Vec<u8> {
    resources::DEFAULT_FONT.to_vec()
}
#[cfg(not(feature = "default_resources"))]
fn get_default_font() -> Vec<u8> {
    Vec::new()
}

/// Manages the window and propagates events to the UI system.
pub struct Window {
    width: f32,
    height: f32,
    dpi: f32,
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
                    .with_vsync(true)
                    .with_srgb(true)
                    .with_gl(gl_request)
                    .with_gl_profile(gl_profile);
                GlWindow::new(window, context, &events_loop)
            };

            if let Ok(result) = create_window(
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

        renderer::initialize_renderer(opengl21, &settings.ui_spritesheet)?;
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

    /// Returns the current status of the mouse. Updated every
    /// `refresh`.
    pub fn get_mouse(&self) -> MouseStatus {
        self.mouse
    }

    /// Re-renders the window, polls for new events and passes them on
    /// to the UI system, and clears the screen with the
    /// `background_*` colors, which consist of 0.0 - 1.0
    /// values. **Note**: Because of vsync, this function will hang
    /// for a while (usually 16ms at max).
    pub fn refresh(
        &mut self,
        background_red: f32,
        background_green: f32,
        background_blue: f32,
    ) -> bool {
        let mut running = true;

        self.frame_timer.end_frame();
        let _ = self.gl_window.swap_buffers();
        unsafe {
            gl::ClearColor(background_red, background_green, background_blue, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.frame_timer.begin_frame();

        let mut resized_logical_size = None;
        let mut mouse_position = None;
        let mut mouse_pressed = None;
        let mut key_inputs = Vec::new();
        let mut characters = Vec::new();
        self.events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => running = false,
                    WindowEvent::Resized(logical_size) => resized_logical_size = Some(logical_size),
                    WindowEvent::CursorMoved { position, .. } => mouse_position = Some(position),
                    WindowEvent::MouseInput { state, .. } => {
                        mouse_pressed = Some(state == ElementState::Pressed)
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(keycode) = input.virtual_keycode {
                            key_inputs.push(KeyStatus {
                                keycode,
                                modifiers: input.modifiers,
                                last_pressed: false,
                                pressed: input.state == ElementState::Pressed,
                            });
                        }
                    }
                    WindowEvent::ReceivedCharacter(c) => characters.push(c),
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
            self.width = logical_size.width as f32;
            self.height = logical_size.height as f32;
            self.dpi = dpi_factor as f32;
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

        ui::update(
            self.width,
            self.height,
            self.dpi,
            self.mouse,
            key_inputs,
            characters,
        );

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
