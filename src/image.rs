use crate::gl;
#[cfg(feature = "png")]
use png;
#[cfg(feature = "png")]
use std::error::Error;

/// Contains the raw pixel color data of an image (`u8` per color
/// channel).
#[derive(Clone, Debug)]
pub struct Image {
    /// The pixels of the image.
    pub pixels: Vec<u8>,
    /// The width of the image.
    pub width: i32,
    /// The height of the image.
    pub height: i32,
    /// The OpenGL format of the image.
    ///
    /// GL_RGBA by default, which means that OpenGL will assume
    /// `pixels` is laid out like so: `[r, g, b, a, r, g, ...]`.
    pub format: u32,
}

impl Image {
    /// Tries to load a PNG image and make an `Image` out of it.
    ///
    /// # Example
    /// ```should_panic
    /// use fae::Image;
    /// use std::fs;
    /// let sprite = Image::from_png(&fs::read("sprite.png").unwrap()).unwrap();
    /// ```
    #[cfg(feature = "png")]
    pub fn from_png(bytes: &[u8]) -> Result<Image, Box<Error>> {
        let decoder = png::Decoder::new(bytes);
        let (info, mut reader) = decoder.read_info()?;
        let mut pixels = vec![0; info.buffer_size()];
        reader.next_frame(&mut pixels)?;
        Ok(Image {
            pixels,
            width: info.width as i32,
            height: info.height as i32,
            format: gl::RGBA,
        })
    }

    /// Creates a solid color image. The color can be 1-4 items
    /// long. If the length of `color` isn't 4, call `format` to set
    /// the appropriate format.
    ///
    /// # Example
    /// ```
    /// use fae::Image;
    /// let image = Image::from_color(128, 128, &[0xB4, 0x6E, 0xC8, 0xFF]);
    /// // image now represents a 128px by 128px image that consists of fully opaque violet pixels.
    /// ```
    pub fn from_color(width: i32, height: i32, color: &[u8]) -> Image {
        let mut pixels = vec![0; (width * height) as usize * color.len()];
        for i in 0..pixels.len() {
            pixels[i] = color[i % color.len()];
        }
        let format = if color.len() == 3 { gl::RGB } else { gl::RGBA };
        Image {
            pixels,
            width,
            height,
            format,
        }
    }

    /// Sets the `internal_format` and `format` parameters given to
    /// glTexImage2D.
    ///
    /// # Example
    /// ```
    /// use fae::{gl, Image};
    /// let image = Image::from_color(128, 128, &[0x88]).format(gl::RED);
    /// // image now represents a 128px by 128px image that consists of half-red pixels taking up only one byte per pixel.
    /// ```
    pub fn format(mut self, format: u32) -> Image {
        self.format = format;
        self
    }
}
