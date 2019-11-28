use crate::api::{DrawCallHandle, GraphicsContext, Image, RectPx, Sprite};

impl DrawCallHandle {
    /// Creates a Sprite struct, which you can render after specifying
    /// your parameters by modifying it.
    ///
    /// Higher Z sprites are drawn over the lower ones (with the
    /// exception of the case described below).
    ///
    /// ## Weird Z-coordinate behavior note
    ///
    /// Try to constrain your z-coordinates to small ranges within
    /// individual draw calls; draw call rendering order is decided by
    /// the highest z-coordinate that each draw call has to draw. This
    /// can even cause visual glitches in alpha-blended draw calls, if
    /// their sprites overlap and have overlapping ranges of
    /// z-coordinates.
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
    ///
    /// # Usage
    /// ```ignore
    /// call.draw(&mut ctx, 0.0)
    ///     .with_coordinates((100.0, 100.0, 16.0, 16.0))
    ///     .with_texture_coordinates((0, 0, 16, 16))
    ///     .finish();
    /// ```
    pub fn draw<'a, 'b>(&'b self, ctx: &'a mut GraphicsContext, z: f32) -> Sprite<'a, 'b> {
        ctx.renderer.draw(&self, z)
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
    /// [`GraphicsContext::upload_texture_region`](struct.GraphicsContext.html#method.upload_texture_region).
    pub fn resize_texture(&self, ctx: &mut GraphicsContext, new_width: i32, new_height: i32) {
        ctx.renderer.resize_texture(self, new_width, new_height);
    }
}
