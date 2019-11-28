use crate::api::GraphicsContext;
use crate::image::Image;
use crate::sprite::Sprite;
use crate::types::RectPx;

pub use crate::renderer::{DrawCallHandle, DrawCallParameters};

impl DrawCallHandle {
    /// Creates a new draw call you can draw with.
    ///
    /// `DrawCallParameters` implements `Default`, so the idea is that
    /// you specify the parts you want to customize. For basic colored
    /// rectangle rendering, the default configuration is all you
    /// need.
    pub fn create(ctx: &mut GraphicsContext, params: DrawCallParameters) -> DrawCallHandle {
        ctx.renderer.create_draw_call(params)
    }

    /// Creates a Sprite struct, which you can draw by calling
    /// `.finish()`. The parameters are set using [the builder
    /// pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html).
    ///
    /// # Usage
    /// ```ignore
    /// draw_call_handle.draw(&mut ctx)
    ///     .with_coordinates((100.0, 100.0, 16.0, 16.0))
    ///     .with_texture_coordinates((0, 0, 16, 16))
    ///     .finish();
    /// ```
    ///
    /// ## Optimization tips
    /// - Draw the sprites in front first. This way you'll avoid
    ///   rendering over already drawn pixels. If you're rendering
    ///   *lots* of sprites, this is a good place to start optimizing.
    /// - If possible, make your textures without using alpha values
    ///   between 1 and 0 (ie. use only 100% and 0% opacity), and
    ///   disable `alpha_blending` in your draw call. These kinds of
    ///   sprites can be drawn much more efficiently when it comes to
    ///   overdraw.
    pub fn draw<'a, 'b>(&'b self, ctx: &'a mut GraphicsContext) -> Sprite<'a, 'b> {
        ctx.renderer.draw(&self)
    }

    /// Upload an image into the specified region in a draw call's
    /// texture.
    ///
    /// If the width and height of `region` and `image` don't match,
    /// or the `region` isn't completely contained within the texture,
    /// this function will do nothing and return false.
    ///
    /// See also:
    /// [`Image::create_null`](struct.Image.html#method.create_null).
    pub fn upload_texture_region(
        &self,
        ctx: &mut GraphicsContext,
        region: RectPx,
        image: &Image,
    ) -> bool {
        ctx.renderer.upload_texture_region(self, region, image)
    }

    /// Resize the draw call's texture to a new width and height,
    /// which must be equal or greater than the original
    /// dimensions. The previous contents of the texture are preserved
    /// in the origin corner of the texture.
    ///
    /// If `new_width` is less than the current width, or `new_height`
    /// is less than the current height, this function will do
    /// nothing.
    ///
    /// See also:
    /// [`DrawCallHandle::upload_texture_region`](struct.DrawCallHandle.html#method.upload_texture_region).
    pub fn resize_texture(&self, ctx: &mut GraphicsContext, new_width: i32, new_height: i32) {
        ctx.renderer.resize_texture(self, new_width, new_height);
    }
}
