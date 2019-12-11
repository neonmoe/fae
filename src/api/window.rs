use crate::api::GraphicsContext;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, NotCurrent};

use crate::error::GlutinError;

pub use glutin;

/// Wrapper for Glutin window creation.
///
/// This wrapper handles the OpenGL context, passes relevant events to
/// fae, and does the appropriate preparation and teardown when
/// rendering.
pub struct Window<T: 'static> {
    event_loop: EventLoop<T>,
    ctx: GraphicsContext,
}

impl Window<()> {
    /// Creates a new `Window`.
    ///
    /// # Errors
    ///
    /// See the [`GlutinError`](enum.GlutinError.html) variants.
    pub fn new(
        (window_builder, context_builder): (WindowBuilder, ContextBuilder<'_, NotCurrent>),
    ) -> Result<Window<()>, GlutinError> {
        Window::with_event_loop(EventLoop::new(), (window_builder, context_builder))
    }
}

impl<T> Window<T> {
    /// Creates a new `Window` with an event loop provided by the
    /// user.
    ///
    /// # Errors
    ///
    /// See the [`GlutinError`](enum.GlutinError.html) variants.
    pub fn with_event_loop(
        event_loop: EventLoop<T>,
        (window_builder, context_builder): (WindowBuilder, ContextBuilder<'_, NotCurrent>),
    ) -> Result<Window<T>, GlutinError> {
        let context = context_builder.build_windowed(window_builder, &event_loop)?;
        let context = unsafe {
            let context = context.make_current()?;
            crate::gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
            context
        };

        Ok(Window {
            event_loop,
            ctx: GraphicsContext::new(context),
        })
    }

    /// Returns the rendering context of this window.
    pub fn ctx(&mut self) -> &mut GraphicsContext {
        &mut self.ctx
    }

    /// Starts the event loop.
    ///
    /// This is a wrapper for
    /// [`winit::event_loop::EventLoop::run`](https://docs.rs/winit/0.20.0-alpha4/winit/event_loop/struct.EventLoop.html#method.run),
    /// and it is used in a similar fashion. Whatever events are
    /// needed to keep the graphics flowing are intercepted, handled,
    /// and passed on to `event_handler` for your use.
    ///
    /// When handling the EventsCleared event, the rendering context
    /// is set up for drawing. This is when the first parameter of `F`
    /// is Some, and this should be considered the event during which
    /// you should update and render. Consider the following
    /// comparison:
    ///
    /// ```ignore
    /// // A traditional game loop:
    /// loop {
    ///     handle_input(); // Polls input at the start of every frame
    ///     update();
    ///     render();
    ///     swap_buffers();
    /// }
    /// ```
    ///
    /// ```ignore
    /// // A fae (/winit) game loop:
    /// window.run(|ctx, event, _, _| {
    ///     if let Some(ctx) = ctx {
    ///         update();
    ///         render(ctx);
    ///     } else {
    ///         handle_input(event); // Handles input events as they come
    ///     }
    /// });
    /// // Swapping buffers (and rendering, actually) is done when winit wants, by Window::run
    /// ```
    pub fn run<F>(self, mut event_handler: F) -> !
    where
        F: 'static
            + FnMut(
                Option<&mut GraphicsContext>,
                Event<T>,
                &EventLoopWindowTarget<T>,
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
