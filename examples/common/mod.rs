#![allow(dead_code)]

use fae::glutin::dpi::*;
use fae::glutin::window::WindowBuilder;
use fae::glutin::*;

#[cfg(feature = "text")]
pub use create_font::*;
#[cfg(feature = "text")]
mod create_font {
    use fae::{FontHandle, GraphicsContext};

    cfg_if::cfg_if! {
        if #[cfg(feature = "ttf")] {
            pub fn create_font(ctx: &mut GraphicsContext) -> FontHandle {
                use font_loader::system_fonts;
                let property = system_fonts::FontPropertyBuilder::new()
                    .build();
                let (font_bytes, _) = system_fonts::get(&property).unwrap();
                FontHandle::with_ttf(ctx, font_bytes).unwrap()
            }
        } else if #[cfg(feature = "font8x8")] {
            pub fn create_font(ctx: &mut GraphicsContext) -> FontHandle {
                FontHandle::with_font8x8(ctx, true)
            }
        } else {
            pub fn create_font(_ctx: &mut GraphicsContext) -> FontHandle {
                panic!("no font feature (`font8x8` or `ttf`) enabled")
            }
        }
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
            .with_inner_size(LogicalSize::new(
                f64::from(settings.width),
                f64::from(settings.height),
            ))
            .with_visible(false);
        let context = ContextBuilder::new()
            .with_vsync(settings.vsync)
            .with_srgb(true)
            .with_multisampling(settings.multisample)
            .with_gl(GlRequest::Latest);
        (window, context)
    }
}
