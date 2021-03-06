#![allow(dead_code)]

#[cfg(feature = "text")]
pub use create_font::*;
#[cfg(feature = "text")]
mod create_font {
    use fae::{Context, Font};

    cfg_if::cfg_if! {
        if #[cfg(feature = "ttf")] {
            pub fn create_font(ctx: &mut Context) -> Font {
                use font_loader::system_fonts;
                let property = system_fonts::FontPropertyBuilder::new()
                    .build();
                let (font_bytes, _) = system_fonts::get(&property).unwrap();
                Font::with_ttf(ctx, font_bytes).unwrap()
            }
        } else if #[cfg(feature = "font8x8")] {
            pub fn create_font(ctx: &mut Context) -> Font {
                Font::with_font8x8(ctx, true)
            }
        } else {
            pub fn create_font(_ctx: &mut Context) -> Font {
                panic!("no font feature (`font8x8` or `ttf`) enabled")
            }
        }
    }
}

use std::time::{Duration, Instant};
pub struct FpsCounter {
    timestamps: Vec<Instant>,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            timestamps: Vec::new(),
        }
    }

    pub fn record_frame(&mut self) {
        self.timestamps.push(Instant::now());
    }

    pub fn get_fps(&mut self) -> usize {
        let second_ago = Instant::now() - Duration::from_secs(1);
        self.timestamps.retain(|timestamp| *timestamp > second_ago);
        self.timestamps.len()
    }
}
