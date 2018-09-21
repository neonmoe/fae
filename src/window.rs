use gl;
use glutin::dpi::*;
use glutin::*;
use renderer;
use std::env;
use std::error::Error;
use ui::{self, MouseStatus};

pub struct Window {
    pub width: f64,
    pub height: f64,
    gl_window: GlWindow,
    events_loop: EventsLoop,
    mouse: MouseStatus,
}

impl Window {
    pub fn new(title: &str, logical_width: f64, logical_height: f64) -> Result<Window, Box<Error>> {
        // FIXME: Enable wayland support by not setting the backend to
        // x11 automatically. Note: At the time of writing, wayland
        // support in winit seems to be buggy. At the very least, it
        // doesn't seem to work with the sway the
        env::set_var("WINIT_UNIX_BACKEND", "x11");

        let events_loop = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(LogicalSize::new(logical_width, logical_height));
        let context = ContextBuilder::new()
            .with_vsync(true)
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core);
        let gl_window = GlWindow::new(window, context, &events_loop)?;

        unsafe {
            gl_window.make_current()?;
        }

        unsafe {
            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.85, 0.85, 0.85, 1.0);
        }

        renderer::initialize()?;
        renderer::initialize_font(include_bytes!("fonts/FiraSans.ttf"))?;

        Ok(Window {
            width: logical_width,
            height: logical_height,
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
        }

        if let Some(position) = mouse_position {
            self.mouse.x = position.x as f32;
            self.mouse.y = position.y as f32;
        }

        self.mouse.last_pressed = self.mouse.pressed;
        if let Some(pressed) = mouse_pressed {
            self.mouse.pressed = pressed;
        }

        ui::update(self.width, self.height, self.mouse);

        running
    }
}
