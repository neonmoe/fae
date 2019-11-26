#![allow(dead_code)]

use fae::glutin::dpi::*;
use fae::glutin::*;

#[cfg(feature = "text")]
pub use renderer_and_text_renderer_creation::*;
#[cfg(feature = "text")]
mod renderer_and_text_renderer_creation {
    use fae::text::TextRenderer;
    use fae::{Renderer, Window};

    cfg_if::cfg_if! {
        if #[cfg(feature = "ttf")] {
            fn create_text_renderer(renderer: &mut Renderer) -> TextRenderer {
                use font_loader::system_fonts;
                let property = system_fonts::FontPropertyBuilder::new()
                    .build();
                let (font_bytes, _) = system_fonts::get(&property).unwrap();
                TextRenderer::with_ttf(renderer, font_bytes).unwrap()
            }
        } else if #[cfg(feature = "font8x8")] {
            fn create_text_renderer(renderer: &mut Renderer) -> TextRenderer {
                TextRenderer::with_font8x8(renderer, true)
            }
        } else {
            fn create_text_renderer(_renderer: &mut Renderer) -> TextRenderer {
                panic!("no font feature (`font8x8` or `ttf`) enabled")
            }
        }
    }

    pub fn create_renderers(window: &Window) -> (Renderer, TextRenderer) {
        let mut renderer = Renderer::new(&window);
        let text = create_text_renderer(&mut renderer);
        (renderer, text)
    }
}

pub struct WindowSettings {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub vsync: bool,
    pub multisample: u16,
}

impl Default for WindowSettings {
    fn default() -> WindowSettings {
        WindowSettings {
            title: std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(std::ffi::OsStr::to_os_string))
                .and_then(|s| s.into_string().ok())
                .unwrap_or_default(),
            width: 640.0,
            height: 480.0,
            vsync: true,
            multisample: 4,
        }
    }
}

impl<'a> From<WindowSettings> for (WindowBuilder, ContextBuilder<'a, NotCurrent>) {
    fn from(settings: WindowSettings) -> (WindowBuilder, ContextBuilder<'a, NotCurrent>) {
        let window = WindowBuilder::new()
            .with_title(settings.title.clone())
            .with_dimensions(LogicalSize::new(
                f64::from(settings.width),
                f64::from(settings.height),
            ))
            .with_visibility(false);
        let context = ContextBuilder::new()
            .with_vsync(settings.vsync)
            .with_srgb(true)
            .with_multisampling(settings.multisample)
            .with_gl(GlRequest::Latest);
        (window, context)
    }
}
