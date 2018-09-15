use gl;
use glutin::dpi::*;
use glutin::*;
use std::env;

pub struct Window {
    pub width: f64,
    pub height: f64,
    gl_window: GlWindow,
    events_loop: EventsLoop,
}

impl Window {
    pub fn new(title: &str, logical_width: f64, logical_height: f64) -> Window {
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
        let gl_window = GlWindow::new(window, context, &events_loop).unwrap();

        unsafe {
            gl_window.make_current().unwrap();
        }

        unsafe {
            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);

            // Print OpenGL version
            // TODO:
            println!(
                "OpenGL version: {:?}",
                ::std::ffi::CStr::from_ptr(::std::mem::transmute::<*const u8, *const i8>(
                    gl::GetString(gl::VERSION)
                ))
            );
        }

        Window {
            width: logical_width,
            height: logical_height,
            gl_window,
            events_loop,
        }
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
        self.events_loop.poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => running = false,
                WindowEvent::Resized(logical_size) => {
                    resized_logical_size = Some(logical_size);
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

        running
    }
}
