//! Default resources, included with the default feature
//! `default_resources`.

use png;
use std::error::Error;
use std::io::Read;

/// Default spritesheet for the GUI elements.
#[cfg(feature = "default_resources")]
pub static DEFAULT_UI_SPRITESHEET: &'static [u8] = include_bytes!("gui.png");
/// Default font, Fira Sans.
#[cfg(feature = "default_resources")]
pub static DEFAULT_FONT: &'static [u8] = include_bytes!("FiraSans.ttf");

/// Contains the raw pixel color data of an image in RGBA format (u8
/// per color channel)x.
pub struct Image {
    /// The pixels of the image.
    pub pixels: Vec<u8>,
    /// The width of the image.
    pub width: i32,
    /// The height of the image.
    pub height: i32,
}

/// Tries to load a PNG image and make an `Image` out of it.
pub fn load_png<R: Read>(bytes: R) -> Result<Image, Box<Error>> {
    let decoder = png::Decoder::new(bytes);
    let (info, mut reader) = decoder.read_info()?;
    let mut pixels = vec![0; info.buffer_size()];
    reader.next_frame(&mut pixels)?;
    Ok(Image {
        pixels,
        width: info.width as i32,
        height: info.height as i32,
    })
}
