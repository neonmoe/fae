use crate::gl_version::OpenGlVersion;
use crate::renderer::Renderer;
#[cfg(feature = "text")]
use crate::text::TextRenderer;

/// The overarching state of the crate. Intended to live outside of
/// the main game loop.
///
/// This is the struct you can get a GraphicsContext from, and which
/// should live as long as you're drawing anything. Illustrated:
/// ```no_run
/// use fae::Context;
/// let mut fae_context: Context = Context::new();
/// # let (width, height, dpi_factor) = (0.0, 0.0, 0.0);
/// # let spritesheet = fae::SpritesheetBuilder::default().build(&mut fae_context);
///
/// loop {
///     // Here's your gameloop, and now you want to draw something.
///
///     // First, create the GraphicsContext with start_frame.
///     let mut ctx: fae::GraphicsContext = fae_context.start_frame(width, height, dpi_factor);
///
///     // Then do your rendering stuff.
///     spritesheet.draw(&mut ctx)
///         /* ... */
///         .finish();
///
///     // Finish frame and consume the GraphicsContext.
///     ctx.finish_frame();
///
///     // swap buffers, fae_context.synchronize(), etc.
/// }
/// ```
///
/// This construct makes the state of fae more clear, as you can only
/// have access to either the Context or the GraphicsContext, as
/// well as providing a good synchronization point (start_frame) where
/// the window's state is passed to fae, ensuring that all rendering
/// operations are done based on up-to-date information.
pub struct Context {
    pub(crate) renderer: Renderer,
    #[cfg(feature = "text")]
    pub(crate) text_renderers: Vec<TextRenderer>,
}

impl Context {
    /// Creates a new GraphicsContext. See the Safety section.
    ///
    /// # Safety
    ///
    /// Basically everything in fae assumes that it can call OpenGL,
    /// so please ensure you have called something along the lines of:
    ///
    /// ```ignore
    /// unsafe { fae::gl::load_with(|symbol| context.get_proc_address(symbol) as *const _); }
    /// ```
    ///
    /// Before creating a Context.
    ///
    /// The width, height and dpi_factor are only initial values; they
    /// are updated in the call to
    /// [`Context::start_frame()`](struct.Context.html#method.start_frame).
    pub fn new() -> Context {
        Context {
            renderer: Renderer::new(),
            #[cfg(feature = "text")]
            text_renderers: Vec::new(),
        }
    }

    /// Returns true when running in legacy mode (OpenGL 3.3+
    /// optimizations off).
    pub fn is_legacy(&self) -> bool {
        self.renderer.legacy
    }

    /// Returns the OpenGL version if it could be parsed.
    pub fn get_opengl_version(&self) -> &OpenGlVersion {
        &self.renderer.version
    }

    /// Tries to ensure that all the commands queued in the GPU have been processed.
    ///
    /// Call this after swap_buffers to ensure that everything after
    /// this happens only after the frame has been sent to the screen,
    /// but don't trust this to actually work. Doing vsync properly
    /// with OpenGL is a mess, as far as I know.
    pub fn synchronize(&mut self) {
        self.renderer.synchronize();
    }

    /// Creates a GraphicsContext for this frame.
    ///
    /// The parameters `width` and `height` are the dimensions of the
    /// window, and dpi_factor is a multiplier, such that: `width *
    /// dpi_factor` is the window's width in physical pixels, and
    /// `height * dpi_factor` is the height in physical pixels.
    pub fn start_frame(&mut self, width: f32, height: f32, dpi_factor: f32) -> GraphicsContext {
        self.renderer.prepare_new_frame(dpi_factor);

        #[cfg(feature = "text")]
        for font in &mut self.text_renderers {
            font.prepare_new_frame(&mut self.renderer, dpi_factor, width, height);
        }

        GraphicsContext {
            renderer: &mut self.renderer,
            #[cfg(feature = "text")]
            text_renderers: &mut self.text_renderers,
            width,
            height,
            dpi_factor,
        }
    }

    /// Renders the frame with the given `width` and `height`.
    ///
    /// See
    /// [`Context::start_frame`](struct.Context.html#method.start_frame)
    /// for more information on what `width` and `height` are,
    /// specifically.
    ///
    /// This should generally be called after
    /// [`GraphicsContext::finish_frame`](struct.GraphicsContext.html#method.finish_frame),
    /// but can also be used to redraw the previous frame.
    pub fn render(&mut self, width: f32, height: f32) {
        self.renderer.render(width, height);
    }
}

/// Draw stuff on the screen with this.
///
/// Create this struct with
/// [`Context::start_frame()`](struct.Context.html#method.start_frame).
///
/// Then, pass it to:
/// - [`Spritesheet::draw`](struct.Spritesheet.html#method.draw) to draw sprites,
/// - [`Font::draw`](struct.Font.html#method.draw) to draw text.
///
/// And after doing all the drawing, call
/// [`GraphicsContext::finish_frame()`](struct.GraphicsContext.html#method.finish_frame)
/// to flush all the rendering operations.
pub struct GraphicsContext<'a> {
    pub(crate) renderer: &'a mut Renderer,
    #[cfg(feature = "text")]
    pub(crate) text_renderers: &'a mut Vec<TextRenderer>,

    /// The width of the window in logical coordinates. Multiply with
    /// `dpi_factor` to get the width in physical pixels.
    pub width: f32,
    /// The height of the window in logical coordinates. Multiply with
    /// `dpi_factor` to get the height in physical pixels.
    pub height: f32,
    /// The dpi multiplier of the window.
    pub dpi_factor: f32,
}

impl GraphicsContext<'_> {
    /// Consume this GraphicsContext to render everything that has
    /// been queued with `draw` calls so far. Call
    /// [`Context::render()`](struct.Context.html#method.render)
    /// and swap buffers after this.
    pub fn finish_frame(self) {
        #[cfg(feature = "text")]
        for font in self.text_renderers {
            font.compose_draw_call(self.renderer);
        }
        self.renderer.finish_frame();
    }
}
