#[cfg(feature = "png")]
use crate::error::ImageLoadingError;
use crate::gl;
use crate::gl::types::*;
#[cfg(feature = "png")]
use png;

/// Contains the raw pixel color data of an image.
///
/// See also: [`DrawCallParameters`](struct.DrawCallParameters.html).
#[derive(Clone, Debug)]
pub struct Image {
    /// The pixels of the image.
    pub pixels: Vec<u8>,
    /// The width of the image.
    pub width: i32,
    /// The height of the image.
    pub height: i32,
    /// The OpenGL format of the image.
    pub format: GLuint,
    /// The OpenGL type of the pixels of the image.
    pub(crate) pixel_type: GLuint,
    /// Whether the image represents a null pointer for
    /// glTexImage2D. If true, the memory for the texture of width x
    /// height will be allocated, but will probably be garbage.
    pub(crate) null_data: bool,
}

impl Image {
    /// Parses a PNG image and makes an `Image` out of it.
    ///
    /// This function assumes that the `Image` is in SRGB space. If
    /// you want to change this, use
    /// [`Image::with_format`](struct.Image.html#method.with_format).
    ///
    /// # Errors
    ///
    /// A [`PngError`](enum.ImageLoadingError.html#variant.PngError)
    /// will be returned if the data couldn't be read for some reason
    /// by the `png` crate (most probably, `bytes` doesn't describe a
    /// valid PNG image). An
    /// [`UnsupportedFormat`](enum.ImageLoadingError.html#variant.UnsupportedFormat)
    /// error will be returned if the PNG isn't in either RGB or RGBA
    /// space. An
    /// [`UnsupportedBitDepth`](enum.ImageLoadingError.html#variant.UnsupportedBitDepth)
    /// error will be returned if the PNG bit depth isn't 8 or 16 bits
    /// per channel.
    ///
    /// # Example
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let sprite = fae::Image::with_png(&std::fs::read("sprite.png")?)?;
    /// # Ok(()) }
    /// ```
    #[cfg(feature = "png")]
    pub fn with_png(bytes: &[u8]) -> Result<Image, ImageLoadingError> {
        use png::{BitDepth, ColorType, Decoder};
        let decoder = Decoder::new(bytes);
        let (info, mut reader) = decoder.read_info()?;
        let format = match info.color_type {
            ColorType::RGB => gl::SRGB,
            ColorType::RGBA => gl::SRGB_ALPHA,
            format => return Err(ImageLoadingError::UnsupportedFormat(format)),
        };
        let pixel_type = match info.bit_depth {
            BitDepth::Eight => gl::UNSIGNED_BYTE,
            BitDepth::Sixteen => gl::UNSIGNED_SHORT,
            bitdepth => return Err(ImageLoadingError::UnsupportedBitDepth(bitdepth)),
        };
        let mut pixels = vec![0; info.buffer_size()];
        reader.next_frame(&mut pixels)?;
        Ok(Image {
            pixels,
            width: info.width as i32,
            height: info.height as i32,
            format,
            pixel_type,
            null_data: false,
        })
    }

    /// Creates a solid color image.
    ///
    /// The color can be 1-4 items long, and will be interpreted as
    /// being in SRGB color space. If the length of `color` isn't 4,
    /// or you wish to enter a color in linear color space, use
    /// [`with_format`](struct.Image.html#method.with_format).
    ///
    /// The color is interpreted as SRGB by default to be consistent
    /// with loading images from the disk, which are assumed to be in
    /// SRGB space by default.
    ///
    /// # Example
    /// ```
    /// use fae::Image;
    /// let image = Image::with_color(128, 128, &[0xB4, 0x6E, 0xC8, 0xFF]);
    /// // image now represents a 128px by 128px image that consists of fully opaque violet pixels.
    /// ```
    pub fn with_color(width: i32, height: i32, color: &[u8]) -> Image {
        let mut pixels = vec![0; (width * height) as usize * color.len()];
        for i in 0..pixels.len() {
            pixels[i] = color[i % color.len()];
        }
        let format = if color.len() == 3 {
            gl::SRGB
        } else {
            gl::SRGB_ALPHA
        };
        Image {
            pixels,
            width,
            height,
            format,
            pixel_type: gl::UNSIGNED_BYTE,
            null_data: false,
        }
    }

    /// Creates an image with a specified width, height and a format,
    /// and signals to OpenGL that the texture will be filled in
    /// later. The memory for the texture will be allocated on the
    /// GPU, but no pixel data needs to be sent from the CPU to the
    /// GPU during initialization.
    ///
    /// See also:
    /// [`DrawCallHandle::upload_texture_region`](struct.DrawCallHandle.html#method.upload_texture_region).
    pub fn with_null_texture(width: i32, height: i32, format: GLuint) -> Image {
        Image {
            pixels: Vec::new(),
            width,
            height,
            format,
            pixel_type: gl::UNSIGNED_BYTE,
            null_data: true,
        }
    }

    /// Sets the format of the pixels.
    ///
    /// Generally: `gl::RED` for grayscale pixels, `gl::RGB` for
    /// linear non-transparent pixels, and `gl::RGBA` for linear and
    /// transparent pixels.
    ///
    /// # Example
    /// ```
    /// use fae::{gl, Image};
    /// let image = Image::with_color(128, 128, &[0x88]).format(gl::RED);
    /// // image now represents a 128px by 128px image that consists of half-red pixels taking up only one byte per pixel.
    /// ```
    pub fn format<'a>(&'a mut self, format: GLuint) -> &'a mut Self {
        self.format = format;
        self
    }
}
