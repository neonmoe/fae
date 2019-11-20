use fae::{text::TextRenderer, Renderer, Window};

cfg_if::cfg_if! {
    if #[cfg(feature = "rusttype")] {
        fn create_text_renderer(renderer: &mut Renderer) -> TextRenderer {
            use font_loader::system_fonts;
            let property = system_fonts::FontPropertyBuilder::new()
                .family("serif")
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
            panic!("no font feature (`font8x8` or `rusttype`) enabled")
        }
    }
}

pub fn create_renderers(window: &Window) -> (Renderer, TextRenderer) {
    let mut renderer = Renderer::new(&window);
    let text = create_text_renderer(&mut renderer);
    (renderer, text)
}
