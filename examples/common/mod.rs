use fae::{text::TextRenderer, Renderer, Window};

pub fn create_renderers(window: &Window) -> (Renderer, TextRenderer) {
    let mut renderer = Renderer::new(&window);
    #[cfg(all(feature = "font8x8", not(feature = "rusttype")))]
    let text = TextRenderer::with_font8x8(&mut renderer, true);
    #[cfg(feature = "rusttype")]
    let text = TextRenderer::with_ttf(
        &mut renderer,
        include_bytes!("../res/FiraSans.ttf").to_vec(),
    );
    (renderer, text)
}
