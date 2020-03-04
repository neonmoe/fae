use crate::api::{Context, GraphicsContext};
use crate::image::Image;
use crate::sprite::Sprite;
use crate::types::{Rect, RectPx};

use crate::renderer::{DrawCallHandle, Shaders, TextureWrapping};

/// Holds a texture for rendering.
///
/// This struct is safe to clone in order to use elsewhere: the only
/// data held by this struct is a handle to the internal draw
/// call. Currently, it is not possible to delete draw calls, and the
/// only way to create a new one is by using the
/// [`SpritesheetBuilder`](struct.SpritesheetBuilder.html).
///
///
/// # Example
///
/// ```no_run
/// # let an_image = fae::Image::with_null_texture(16, 16, fae::gl::RED);
/// let mut ctx = fae::Context::new();
/// use fae::SpritesheetBuilder;
///
/// // Creating the spritesheet:
/// let spritesheet = SpritesheetBuilder::default()
///     .image(an_image)
///     .alpha_blending(false)
///     .build(&mut ctx);
///
/// // Later, in rendering code:
/// # let (width, height, dpi_factor) = (0.0, 0.0, 0.0);
/// let mut ctx = ctx.start_frame(width, height, dpi_factor);
/// spritesheet.draw(&mut ctx)
///     .coordinates((100.0, 100.0, 16.0, 16.0))
///     .texture_coordinates((0, 0, 16, 16))
///     .finish();
/// ```
#[derive(Clone, Debug)]
pub struct Spritesheet {
    pub(crate) handle: DrawCallHandle,
}

impl Spritesheet {
    /// Creates a Sprite struct, a builder struct which defines the
    /// action "draw a sprite from this spritesheet onto the screen."
    /// To "execute" the builder, call
    /// [`finish()`](struct.Sprite.html#method.finish). The parameters
    /// are set using [the builder
    /// pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html).
    ///
    /// ## Optimization tips
    ///
    /// - If possible, make your textures without using alpha values
    /// between 1 and 0 (ie. use only 100% and 0% opacity), and
    /// disable `alpha_blending` in
    /// [`SpritesheetBuilder`](struct.SpritesheetBuilder.html#method.alpha_blending). These
    /// kinds of sprites can be drawn much more efficiently when it
    /// comes to overdraw.
    ///
    /// - If `alpha_blending` is disabled, draw the sprites in front
    /// first. This way you'll avoid rendering over already drawn
    /// pixels. If you're rendering *lots* of sprites, this is a good
    /// place to start optimizing.
    ///
    ///   - Note: if `alpha_blending` is *enabled*, the you should
    ///   draw the sprites in the *back* first, to ensure correct
    ///   rendering.
    pub fn draw<'a, 'b>(&'b self, ctx: &'a mut GraphicsContext) -> Sprite<'a, 'b> {
        ctx.renderer.draw(&self.handle)
    }

    // TODO(0.6.0): Add a function to render inside a spritesheet in when not in legacy mode

    /// Upload an image into the specified region in the spritesheet.
    ///
    /// As the inner values of `region` will be floored before use, it
    /// is recommended to use a `(i32, i32, i32, i32)` as the `region`
    /// parameter to ensure expected behavior.
    ///
    /// If the width and height of `region` and `image` don't match,
    /// or the `region` isn't completely contained within the texture,
    /// this function will do nothing and return false.
    ///
    /// See also:
    /// [`Image::with_null_texture`](struct.Image.html#method.with_null_texture).
    pub fn upload_texture_region<R: Into<Rect>>(
        &self,
        ctx: &mut GraphicsContext,
        region: R,
        image: &Image,
    ) -> bool {
        let Rect {
            x,
            y,
            width,
            height,
        } = region.into();
        let region = RectPx {
            x: x.floor() as i32,
            y: y.floor() as i32,
            width: width.floor() as i32,
            height: height.floor() as i32,
        };
        ctx.renderer
            .upload_texture_region(&self.handle, region, image)
    }

    /// Resize the spritesheet texture to a new width and height,
    /// which must be equal or greater than the original
    /// dimensions. The previous contents of the texture are preserved
    /// in the origin corner of the texture.
    ///
    /// If `new_width` is less than the current width, or `new_height`
    /// is less than the current height, this function will do nothing
    /// and return false.
    ///
    /// See also:
    /// [`Spritesheet::upload_texture_region`](struct.Spritesheet.html#method.upload_texture_region).
    pub fn resize_texture(
        &self,
        ctx: &mut GraphicsContext,
        new_width: i32,
        new_height: i32,
    ) -> bool {
        ctx.renderer
            .resize_texture(&self.handle, new_width, new_height)
    }
}

/// Describes how a spritesheet should be blended with the background.
#[derive(Clone, Copy)]
pub struct AlphaBlending {
    /// Whether to blend the sprites. (That is, GL_BLEND.)
    pub blend: bool,
    /// Whether to sort the sprites so that transparency works as
    /// expected.
    pub sort: bool,
}

/// A builder for [`Spritesheet`](struct.Spritesheet.html).
#[derive(Clone)]
pub struct SpritesheetBuilder {
    /// The texture used when drawing with this handle. None can be
    /// used if you want to just draw flat-color quads.
    pub image: Option<Image>,
    /// The shaders used when drawing with this handle.
    pub shaders: Shaders,
    /// Whether to blend with previously drawn pixels when drawing
    /// over them, or just replace the color. Rule of thumb: if the
    /// sprites only use alpha values of 0 and 255 (ie. fully
    /// transparent and fully opaque), set this to false, and true
    /// otherwise. In any case, alpha values of less than 1/256 will
    /// be cut out and won't be rendered at all.
    ///
    /// Internally, this controls whether `GL_BLEND` and back-to-front
    /// sorting are enabled.
    pub alpha_blending: AlphaBlending,
    /// When drawing quads that are smaller than the texture provided,
    /// use linear (true) or nearest neighbor (false) smoothing when
    /// scaling? (Linear is probably always better.)
    pub minification_smoothing: bool,
    /// When drawing quads that are larger than the texture provided,
    /// use linear (true) or nearest neighbor (false) smoothing when
    /// scaling? (Tip: for pixel art or other textures that don't
    /// suffer from jaggies, set this to `false` for the intended
    /// look.)
    pub magnification_smoothing: bool,
    /// Sets the texture's behavior when sampling coordinates under
    /// 0.0 or over 1.0, or smoothing over texture
    /// boundaries. (Corresponds to `GL_TEXTURE_WRAP_S` and
    /// `GL_TEXTURE_WRAP_T`, in that order.)
    pub wrap: (TextureWrapping, TextureWrapping),
    /// Controls whether the colors rendered by this draw call should
    /// be converted into sRGB before display. This should generally
    /// be true, unless you handle gamma in your shaders
    /// yourself. Note that in any case, the fragment shader will
    /// process fragments in linear space: this conversion happens
    /// after blending.
    ///
    /// Internally, this controls whether or not `GL_FRAMEBUFFER_SRGB`
    /// is enabled when drawing with this handle.
    pub srgb: bool,
}

impl Default for SpritesheetBuilder {
    fn default() -> SpritesheetBuilder {
        SpritesheetBuilder {
            image: None,
            shaders: Shaders::default(),
            alpha_blending: AlphaBlending {
                blend: true,
                sort: true,
            },
            minification_smoothing: true,
            magnification_smoothing: true,
            wrap: (TextureWrapping::Clamp, TextureWrapping::Clamp),
            srgb: true,
        }
    }
}

impl SpritesheetBuilder {
    /// Creates a new Spritesheet from this builder.
    pub fn build(&self, ctx: &mut Context) -> Spritesheet {
        Spritesheet {
            handle: ctx.renderer.create_draw_call(
                self.image.as_ref(),
                &self.shaders,
                self.alpha_blending,
                self.minification_smoothing,
                self.magnification_smoothing,
                self.wrap,
                self.srgb,
            ),
        }
    }

    /// Sets the spritesheet's texture.
    pub fn image(&mut self, image: Image) -> &mut SpritesheetBuilder {
        self.image = Some(image);
        self
    }

    /// Sets the spritesheet's shaders.
    pub fn shaders(&mut self, shaders: Shaders) -> &mut SpritesheetBuilder {
        self.shaders = shaders;
        self
    }

    /// Toggles the spritesheet's alpha blending.
    pub fn alpha_blending(&mut self, blend: bool, sort: bool) -> &mut SpritesheetBuilder {
        self.alpha_blending = AlphaBlending { blend, sort };
        self
    }

    /// Sets the spritesheet's minification filter.
    pub fn minification_smoothing(&mut self, smoothing: bool) -> &mut SpritesheetBuilder {
        self.minification_smoothing = smoothing;
        self
    }

    /// Sets the spritesheet's magnification filter.
    pub fn magnification_smoothing(&mut self, smoothing: bool) -> &mut SpritesheetBuilder {
        self.magnification_smoothing = smoothing;
        self
    }

    /// Sets the spritesheet texture's wrapping behavior.
    pub fn wrapping_behavior(
        &mut self,
        wrap_s: TextureWrapping,
        wrap_t: TextureWrapping,
    ) -> &mut SpritesheetBuilder {
        self.wrap = (wrap_s, wrap_t);
        self
    }

    /// Toggles the srgb-ness of the draw call.
    pub fn srgb(&mut self, srgb: bool) -> &mut SpritesheetBuilder {
        self.srgb = srgb;
        self
    }
}
