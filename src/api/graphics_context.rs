use crate::gl_version::OpenGlVersion;
use crate::renderer::Renderer;
#[cfg(feature = "text")]
use crate::text::TextRenderer;

use glutin::dpi::LogicalSize;
use glutin::{PossiblyCurrent, WindowedContext};

/// Draw stuff on the screen with this.
///
/// Borrow this struct from:
/// - [`Window::ctx`](struct.Window.html#method.ctx) before starting
///   the event loop,
/// - The first parameter of the closure in
///   [`Window::run`](struct.Window.html#method.run) (during the event
///   loop).
///
/// Then, pass it to:
/// - [`Spritesheet::draw`](struct.Spritesheet.html#method.draw) to draw sprites,
/// - [`Font::draw`](struct.Font.html#method.draw) to draw text.
pub struct GraphicsContext {
    window: Option<WindowedContext<PossiblyCurrent>>,
    env_dpi_factor: f32,

    pub(crate) renderer: Renderer,
    #[cfg(feature = "text")]
    pub(crate) text_renderers: Vec<TextRenderer>,

    /// The width of the window in logical coordinates. Multiply with
    /// `dpi_factor` to get the width in physical pixels.
    pub width: f32,
    /// The height of the window in logical coordinates. Multiply with
    /// `dpi_factor` to get the height in physical pixels.
    pub height: f32,
    /// The dpi multiplier of the window.
    pub dpi_factor: f32,
}

impl GraphicsContext {
    pub(crate) fn new(context: WindowedContext<PossiblyCurrent>) -> GraphicsContext {
        let env_dpi_factor = {
            let multiplier = get_env_dpi();
            let size = context.window().inner_size();
            let (w, h): (f64, f64) = size.into();
            context
                .window()
                .set_inner_size((w * f64::from(multiplier), h * f64::from(multiplier)).into());
            multiplier
        };

        let size = context.window().inner_size();
        let dpi_factor = env_dpi_factor * context.window().hidpi_factor() as f32;
        let (width, height) = (size.width as f32, size.height as f32);

        GraphicsContext {
            env_dpi_factor,
            window: Some(context),
            renderer: Renderer::new(),
            #[cfg(feature = "text")]
            text_renderers: Vec::new(),
            width,
            height,
            dpi_factor,
        }
    }

    /// Creates a dummy version of the GraphicsContext for no_run
    /// doctest usage. This will cause panics everywhere.
    #[doc(hidden)]
    pub fn dummy() -> GraphicsContext {
        GraphicsContext {
            env_dpi_factor: 1.0,
            window: None,
            renderer: Renderer::new(),
            #[cfg(feature = "text")]
            text_renderers: Vec::new(),
            width: 0.0,
            height: 0.0,
            dpi_factor: 1.0,
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

    /// Returns the glutin context.
    pub fn glutin_context(&self) -> &WindowedContext<PossiblyCurrent> {
        self.inner()
    }

    pub(crate) fn swap_buffers(&mut self) {
        let _ = self.inner().swap_buffers();
        self.renderer.synchronize();
    }

    pub(crate) fn render(&mut self) {
        self.renderer.render(self.width, self.height);
    }

    pub(crate) fn resize(&mut self, logical_size: Option<LogicalSize>, dpi_factor: Option<f64>) {
        let logical_size = logical_size.unwrap_or_else(|| self.inner().window().inner_size());
        let dpi_factor = dpi_factor.unwrap_or_else(|| self.inner().window().hidpi_factor());
        let physical_size = logical_size.to_physical(dpi_factor);
        let (width, height): (u32, u32) = physical_size.into();
        unsafe {
            crate::gl::Viewport(0, 0, width as i32, height as i32);
        }
        self.inner().resize(physical_size);
        self.width = logical_size.width as f32 / self.env_dpi_factor;
        self.height = logical_size.height as f32 / self.env_dpi_factor;
        self.dpi_factor = dpi_factor as f32 * self.env_dpi_factor;
    }

    pub(crate) fn prepare_frame(&mut self) {
        self.renderer.prepare_new_frame(self.dpi_factor);

        #[cfg(feature = "text")]
        for font in &mut self.text_renderers {
            font.prepare_new_frame(&mut self.renderer, self.dpi_factor, self.width, self.height);
        }
    }

    pub(crate) fn finish_frame(&mut self) {
        #[cfg(feature = "text")]
        for font in &mut self.text_renderers {
            font.compose_draw_call(&mut self.renderer);
        }
        self.renderer.finish_frame();
        self.inner().window().request_redraw();
    }

    fn inner(&self) -> &WindowedContext<PossiblyCurrent> {
        self.window
            .as_ref()
            .expect("failed to get windowed context; probably using a dummy context")
    }
}

fn get_env_dpi() -> f32 {
    let get_var = |name: &str| {
        std::env::var(name)
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
