//! Window creation utilities for when you don't want to bother
//! writing the glue between `fae` and `glutin`.
use crate::api::GraphicsContext;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, NotCurrent};

use std::error::Error;

pub use glutin;

/// Wrapper for Glutin window creation.
pub struct Window {
    event_loop: EventLoop<()>,
    /// The `Window`'s rendering context.
    pub ctx: GraphicsContext,
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
        let context = unsafe {
            let context = match context.make_current() {
                Ok(current_ctx) => current_ctx,
                Err((_, err)) => return Err(Box::new(err)),
            };
            crate::gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
            context
        };

        Ok(Window {
            event_loop,
            ctx: GraphicsContext::new(context),
        })
    }

    /// Starts the event loop.
    ///
    /// This is a wrapper for
    /// [`winit::event_loop::EventLoop::run`](https://docs.rs/winit/0.20.0-alpha4/winit/event_loop/struct.EventLoop.html#method.run),
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
        let event_loop = self.event_loop;
        let mut ctx = self.ctx;
        event_loop.run(move |event, target, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(logical_size),
                ..
            } => {
                ctx.resize(Some(logical_size), None);
                event_handler(None, event, target, control_flow);
            }
            Event::WindowEvent {
                event: WindowEvent::HiDpiFactorChanged(dpi_factor),
                ..
            } => {
                ctx.resize(None, Some(dpi_factor));
                event_handler(None, event, target, control_flow);
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                ctx.render();
                ctx.swap_buffers();
                event_handler(None, event, target, control_flow);
            }
            Event::EventsCleared => {
                crate::profiler::refresh();
                ctx.prepare_frame();
                event_handler(Some(&mut ctx), event, target, control_flow);
                ctx.finish_frame();
            }
            _ => event_handler(None, event, target, control_flow),
        })
    }
}
