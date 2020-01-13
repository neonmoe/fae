use crate::error::ImageCreationError;
#[cfg(feature = "png")]
use crate::error::PngLoadingError;
use crate::gl;
use crate::gl::types::*;
#[cfg(feature = "png")]
use png;

/// Contains the raw pixel color data of an image.
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
    pub pixel_type: GLuint,
    /// Whether the image represents a null pointer for
    /// glTexImage2D. If true, the memory for the texture of width x
    /// height will be allocated on the GPU, but will probably be
    /// garbage.
    pub null_data: bool,
}

impl Image {
    /// Parses a PNG image and makes an `Image` out of it.
    ///
    /// This function assumes that the image is in SRGB space, so the
    /// image `format` defaults to `SRGB` or `SRGB_ALPHA` if the image
    /// contains the RGB or RGBA components.
    ///
    /// # Color type note
    ///
    /// If your image has a
    /// [`ColorType`](https://docs.rs/png/0.15.2/png/enum.ColorType.html)
    /// of Grayscale, Indexed or GrayscaleAlpha, it will not display
    /// as you would imagine with the default shaders. These images
    /// will use `GL_RED`, `GL_RED`, and `GL_RG` as their format when
    /// uploading the texture, so you need to take that into account
    /// in your shaders (eg. when using GrayscaleAlpha, you'd use the
    /// `color.g` value as your alpha, and `color.r` as your grayscale
    /// value).
    ///
    /// # Errors
    ///
    /// A [`PngError`](enum.PngLoadingError.html#variant.PngError)
    /// will be returned if the data couldn't be read for some reason
    /// by the `png` crate (most probably, `bytes` doesn't describe a
    /// valid PNG image). An
    /// [`UnsupportedBitDepth`](enum.PngLoadingError.html#variant.UnsupportedBitDepth)
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
    pub fn with_png(bytes: &[u8]) -> Result<Image, PngLoadingError> {
        use png::{BitDepth, ColorType, Decoder};
        let decoder = Decoder::new(bytes);
        let (info, mut reader) = decoder.read_info()?;
        let format = match info.color_type {
            ColorType::RGB => gl::SRGB,
            ColorType::RGBA => gl::SRGB_ALPHA,
            ColorType::Grayscale | ColorType::Indexed => gl::RED,
            ColorType::GrayscaleAlpha => gl::RG,
        };
        let pixel_type = match info.bit_depth {
            BitDepth::Eight => gl::UNSIGNED_BYTE,
            BitDepth::Sixteen => gl::UNSIGNED_SHORT,
            bitdepth => return Err(PngLoadingError::UnsupportedBitDepth(bitdepth)),
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
    /// The color can be 1-4 items long, and will be interpreted in
    /// the following order: red, green, blue, alpha.
    ///
    /// The color is interpreted as SRGB when 3 or 4 color components
    /// are provided, to be consistent with loading images from the
    /// disk, which are assumed to be in SRGB space by default.
    ///
    /// Based on the length of the `color` slice, the format of the
    /// resulting image will be `gl::RED`, `gl::RG`, `gl::SRGB`, or
    /// `gl::SRGB_ALPHA`. This can be changed with
    /// [`format()`](struct.Image.html#method.format).
    ///
    /// # Example
    /// ```
    /// use fae::Image;
    /// let image = Image::with_color(128, 128, &[0xB4, 0x6E, 0xC8, 0xFF]);
    /// // image now represents a 128px by 128px image that consists of fully opaque violet pixels.
    /// ```
    pub fn with_color(width: i32, height: i32, color: &[u8]) -> Result<Image, ImageCreationError> {
        let format = match color.len() {
            4 => gl::SRGB_ALPHA,
            3 => gl::SRGB,
            2 => gl::RG,
            1 => gl::RED,
            n => return Err(ImageCreationError::InvalidColorComponentCount(n)),
        };
        let mut pixels = vec![0; (width * height) as usize * color.len()];
        for i in 0..pixels.len() {
            pixels[i] = color[i % color.len()];
        }
        Ok(Image {
            pixels,
            width,
            height,
            format,
            pixel_type: gl::UNSIGNED_BYTE,
            null_data: false,
        })
    }

    /// Creates an image with a specified width, height and a format,
    /// and signals to OpenGL that the texture will be filled in
    /// later. The memory for the texture will be allocated on the
    /// GPU, but no pixel data needs to be sent from the CPU to the
    /// GPU during initialization.
    ///
    /// See also:
    /// [`Spritesheet::upload_texture_region`](struct.Spritesheet.html#method.upload_texture_region).
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
    /// # fn main() -> Result<(), fae::Error> {
    /// let image = Image::with_color(128, 128, &[0x88])?.format(gl::RED);
    /// // image now represents a 128px by 128px image that consists of half-red pixels taking up only one byte per pixel.
    /// # Ok(()) }
    /// ```
    pub fn format(&mut self, format: GLuint) -> &mut Self {
        self.format = format;
        self
    }
}
