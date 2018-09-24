//! Default resources, included with the default feature
//! `default_resources`.

/// Default spritesheet for the GUI elements.
pub static DEFAULT_UI_SPRITESHEET: &'static [u8] = include_bytes!("gui.png");
/// Default font, Fira Sans.
pub static DEFAULT_FONT: &'static [u8] = include_bytes!("FiraSans.ttf");
