//! Window creation utilities for when you don't want to bother
//! writing the glue between `fae` and `glutin`.
use crate::renderer::Renderer;

use glutin::dpi::LogicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::window::WindowBuilder;
use glutin::*;
use std::env;
use std::error::Error;

pub use glutin;

/// Wrapper for Glutin window creation.
pub struct Window {
    event_loop: EventLoop<()>,
    /// The `Window`'s rendering context.
    pub ctx: GraphicsContext,
}

/// The graphics context: used to draw stuff on the screen.
pub struct GraphicsContext {
    env_dpi_factor: f32,
    window: WindowedContext<PossiblyCurrent>,
    renderer: Renderer,
    /// The width of the window in logical coordinates. Multiply with
    /// dpi_factor to get the width in physical pixels.
    pub width: f32,
    /// The height of the window in logical coordinates. Multiply with
    /// dpi_factor to get the height in physical pixels.
    pub height: f32,
    /// The dpi multiplier of the window.
    pub dpi_factor: f32,
}

impl GraphicsContext {
    /// Updates the window (swaps the front and back buffers). The
    /// renderer handle is used for a CPU/GPU synchronization call, so
    /// while it is optional, it's definitely recommended. If vsync is
    /// enabled, this function will hang until the next frame.
    fn swap_buffers(&mut self) {
        let _ = self.window.swap_buffers();
        self.renderer.synchronize();
    }

    /// Returns the renderer.
    pub fn renderer(&mut self) -> &mut Renderer {
        &mut self.renderer
    }
}

impl Window {
    /// Creates a new `Window`.
    ///
    /// Can result in an error if window creation fails or OpenGL
    /// context creation fails.
    pub fn create(
        (window_builder, context_builder): (WindowBuilder, ContextBuilder<'_, NotCurrent>),
    ) -> Result<Window, Box<dyn Error>> {
        let event_loop = EventLoop::new();
        let context = context_builder.build_windowed(window_builder, &event_loop)?;

        let env_dpi_factor = {
            let multiplier = get_env_dpi();
            let size = context.window().inner_size();
            let (w, h): (f64, f64) = size.into();
            context
                .window()
                .set_inner_size((w * f64::from(multiplier), h * f64::from(multiplier)).into());
            multiplier
        };

        let context = unsafe {
            let context = match context.make_current() {
                Ok(current_ctx) => current_ctx,
                Err((_, err)) => return Err(Box::new(err)),
            };
            crate::gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
            context
        };

        let size = context.window().inner_size();
        let (width, height) = (size.width as f32, size.height as f32);
        let renderer = Renderer::new();

        context.window().set_visible(true);

        Ok(Window {
            event_loop,
            ctx: GraphicsContext {
                env_dpi_factor,
                window: context,
                renderer,
                width,
                height,
                dpi_factor: 1.0,
            },
        })
    }

    /// Starts the event loop.
    ///
    /// This is a wrapper for
    /// [`winit::event_loop::EventLoop::run`](https://docs.rs/winit/event_loop/struct.EventLoop.html#method.run),
    /// and it is used in a similar fashion. Whatever events are
    /// needed to keep the graphics flowing are intercepted, handled,
    /// and passed on to `event_handler` for your use.
    ///
    /// When handling the EventsCleared, the rendering context is set
    /// up for drawing. This is when the first parameter of `F` is
    /// Some, and this should be considered the event during which you
    /// should update and render. Consider the following correlation:
    /// ```ignore
    /// // A traditional game loop:
    /// loop {
    ///     handle_input();
    ///     update();
    ///     render();
    ///     swap_buffers();
    /// }
    /// ```
    /// ```ignore
    /// // A fae game loop (a la winit):
    /// window.run(|ctx, event, _, _| {
    ///     if let Some(ctx) = ctx {
    ///         update();
    ///         render();
    ///     } else {
    ///         handle_input(event);
    ///     }
    /// });
    /// // Swapping buffers (and rendering, actually) is done when winit wants, by Window::run
    /// ```
    pub fn run<F>(self, mut event_handler: F) -> !
    where
        F: 'static
            + FnMut(
                Option<&mut GraphicsContext>,
                Event<()>,
                &EventLoopWindowTarget<()>,
                &mut ControlFlow,
            ),
    {
        let handle_resize =
            |ctx: &mut GraphicsContext, logical_size: LogicalSize, dpi_factor: f64| {
                let physical_size = logical_size.to_physical(dpi_factor);

                let (width, height): (u32, u32) = physical_size.into();
                unsafe {
                    crate::gl::Viewport(0, 0, width as i32, height as i32);
                }
                ctx.window.resize(physical_size);
                ctx.width = logical_size.width as f32 / ctx.env_dpi_factor;
                ctx.height = logical_size.height as f32 / ctx.env_dpi_factor;
                ctx.dpi_factor = dpi_factor as f32 * ctx.env_dpi_factor;
            };

        let event_loop = self.event_loop;
        let mut ctx = self.ctx;
        event_loop.run(move |event, target, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(logical_size),
                ..
            } => {
                let dpi_factor = ctx.window.window().hidpi_factor();
                handle_resize(&mut ctx, logical_size, dpi_factor);
                event_handler(None, event, target, control_flow);
            }
            Event::WindowEvent {
                event: WindowEvent::HiDpiFactorChanged(dpi_factor),
                ..
            } => {
                let logical_size = ctx.window.window().inner_size();
                handle_resize(&mut ctx, logical_size, dpi_factor);
                event_handler(None, event, target, control_flow);
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                ctx.renderer.render(ctx.width, ctx.height);
                ctx.swap_buffers();
                event_handler(None, event, target, control_flow);
            }
            Event::EventsCleared => {
                crate::profiler::refresh();
                ctx.renderer.start_frame();
                event_handler(Some(&mut ctx), event, target, control_flow);
                ctx.renderer.finish_frame();
                ctx.window.window().request_redraw();
            }
            _ => event_handler(None, event, target, control_flow),
        })
    }
}

fn get_env_dpi() -> f32 {
    let get_var = |name: &str| {
        env::var(name)
            .ok()
            .and_then(|var| var.parse::<f32>().ok())
            .filter(|f| *f > 0.0)
    };
    if let Some(dpi_factor) = get_var("QT_AUTO_SCREEN_SCALE_FACTOR") {
        return dpi_factor;
    }
    if let Some(dpi_factor) = get_var("QT_SCALE_FACTOR") {
        return dpi_factor;
    }
    if let Some(dpi_factor) = get_var("GDK_SCALE") {
        return dpi_factor;
    }
    if let Some(dpi_factor) = get_var("ELM_SCALE") {
        return dpi_factor;
    }
    1.0
}
