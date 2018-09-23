use gl;
use glutin::dpi::*;
#[cfg(unix)]
use glutin::os::unix::{WindowBuilderExt, XWindowType};
use glutin::*;
use renderer;
#[cfg(feature = "default_resources")]
use resources;
use std::default::Default;
use std::env;
use std::error::Error;
use std::fmt;
use ui::{self, MouseStatus};

pub struct WindowSettings {
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub is_dialog: bool,
    pub ui_spritesheet: Vec<u8>,
    pub font: Vec<u8>,
}

impl Default for WindowSettings {
    fn default() -> WindowSettings {
        WindowSettings {
            title: env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_os_string()))
                .and_then(|s| s.into_string().ok())
                .unwrap_or(String::new()),
            width: 320.0,
            height: 240.0,
            is_dialog: true,
            ui_spritesheet: get_default_resource(Resource::UiSpritesheet),
            font: get_default_resource(Resource::Font),
        }
    }
}

enum Resource {
    UiSpritesheet,
    Font,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Resource::UiSpritesheet => write!(f, "ui_spritesheet"),
            Resource::Font => write!(f, "font"),
        }
    }
}

#[cfg(feature = "default_resources")]
fn get_default_resource(res: Resource) -> Vec<u8> {
    match res {
        Resource::UiSpritesheet => resources::DEFAULT_UI_SPRITESHEET.to_vec(),
        Resource::Font => resources::DEFAULT_FONT.to_vec(),
    }
}
#[cfg(not(feature = "default_resources"))]
fn get_default_resource(res: Resource) -> Vec<u8> {
    panic!(
        "default_resources feature is disabled, but no {} was provided!",
        res
    );
}

pub struct Window {
    pub width: f64,
    pub height: f64,
    gl_window: GlWindow,
    events_loop: EventsLoop,
    mouse: MouseStatus,
}

impl Window {
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

        renderer::initialize(settings.ui_spritesheet)?;
        renderer::initialize_font(settings.font)?;

        Ok(Window {
            width: settings.width,
            height: settings.height,
            gl_window,
            events_loop,
            mouse: MouseStatus {
                x: 0.0,
                y: 0.0,
                last_pressed: false,
                pressed: false,
            },
        })
    }

    pub fn refresh(&mut self) -> bool {
        let mut running = true;
        if let Err(_) = self.gl_window.swap_buffers() {
            running = false;
        }
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let mut resized_logical_size = None;
        let mut mouse_position = None;
        let mut mouse_pressed = None;
        self.events_loop.poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => running = false,
                WindowEvent::Resized(logical_size) => resized_logical_size = Some(logical_size),
                WindowEvent::CursorMoved { position, .. } => mouse_position = Some(position),
                WindowEvent::MouseInput { state, .. } => {
                    mouse_pressed = Some(state == ElementState::Pressed)
                }
                _ => (),
            },
            _ => (),
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

            renderer::update_dpi(dpi_factor as f32);
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

        renderer::render(self.width, self.height);
        ui::update(self.width, self.height, self.mouse);

        running
    }

    #[cfg(unix)]
    fn window_as_dialog(window: WindowBuilder) -> WindowBuilder {
        window.with_x11_window_type(XWindowType::Dialog)
    }
    #[cfg(not(unix))]
    fn window_as_dialog(window: WindowBuilder) -> WindowBuilder {
        window
    }
}
