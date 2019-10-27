use crate::renderer::{DrawCallHandle, Renderer};

/// Contains the parameters needed to draw a quad. Created by
/// [`Renderer::draw()`](struct.Renderer.html#method.draw).
pub struct Renderable<'a, 'b> {
    renderer: &'a mut Renderer,
    call: &'b DrawCallHandle,
    z: f32,
    physical_dimensions: Option<(f32, f32)>,
    coords: (f32, f32, f32, f32),
    texcoords: (f32, f32, f32, f32),
    color: (f32, f32, f32, f32),
    rotation: (f32, f32, f32),
    clip_area: Option<(f32, f32, f32, f32)>,
}

impl<'a, 'b> Renderable<'a, 'b> {
    #[inline]
    pub(crate) fn create(
        renderer: &'a mut Renderer,
        call: &'b DrawCallHandle,
        z: f32,
    ) -> Renderable<'a, 'b> {
        Renderable {
            renderer,
            call,
            z,
            physical_dimensions: None,
            coords: (0.0, 0.0, 0.0, 0.0),
            texcoords: (-1.0, -1.0, -1.0, -1.0),
            color: (1.0, 1.0, 1.0, 1.0),
            rotation: (0.0, 0.0, 0.0),
            clip_area: None,
        }
    }

    /// Renders the quad specified by this struct using the given draw
    /// call and z coordinate. Smaller Z-values are rendered on top.
    #[inline]
    pub fn finish(self) {
        if let Some(area) = self.clip_area {
            self.renderer.draw_quad_clipped(
                self.coords,
                area,
                self.texcoords,
                self.color,
                self.rotation,
                self.z,
                self.call,
            );
        } else {
            self.renderer.draw_quad(
                self.coords,
                self.texcoords,
                self.color,
                self.rotation,
                self.z,
                self.call,
            );
        }
    }

    /// Specifies the screen coordinates (in logical pixels) where the
    /// quad is drawn.
    #[inline]
    pub fn with_coordinates(
        mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Renderable<'a, 'b> {
        self.coords = (x, y, x + width, y + height);
        self.physical_dimensions = Some((
            width * self.renderer.dpi_factor,
            height * self.renderer.dpi_factor,
        ));
        self
    }

    /// Specifies the texture coordinates (in actual pixels, in the
    /// texture's coordinate space) from where the quad is
    /// sampled. NOTE: Specify the screen coordinates (ie. call
    /// `with_coordinates`) before this function to avoid graphical
    /// glitches.
    ///
    /// This function does some correction math to the coordinates. If
    /// you don't really care about pixel-perfectness and/or have
    /// smoothing on, or know what you're doing, `with_uvs` is faster
    /// because it does no additional work.
    #[inline]
    pub fn with_texture_coordinates(
        mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Renderable<'a, 'b> {
        let (tw, th) = self.renderer.get_texture_size(self.call);
        let (tw, th) = (tw as f32, th as f32);
        let (x, y, width, height) = (x as f32, y as f32, width as f32, height as f32);
        if let Some(size) = self.physical_dimensions {
            self.texcoords = (
                x / tw - 0.5 / size.0,
                y / th - 0.5 / size.1,
                (x + width) / tw - 0.5 / size.0,
                (y + height) / th - 0.5 / size.1,
            );
        } else {
            self.texcoords = (x / tw, y / th, (x + width) / tw, (y + height) / th);
        }
        self
    }

    /// Specifies the texture coordinates (as UVs) from where the quad is sampled.
    #[inline]
    pub fn with_uvs(mut self, x0: f32, y0: f32, x1: f32, y1: f32) -> Renderable<'a, 'b> {
        self.texcoords = (x0, y0, x1, y1);
        self
    }

    /// Specifies the clip area. Only the parts that overlap between
    /// the clip area and this quad are rendered.
    #[inline]
    pub fn with_clip_area(mut self, x: f32, y: f32, width: f32, height: f32) -> Renderable<'a, 'b> {
        self.clip_area = Some((x, y, x + width, y + height));
        self
    }

    /// Specifies the color tint of the quad.
    #[inline]
    pub fn with_color(mut self, red: f32, green: f32, blue: f32, alpha: f32) -> Renderable<'a, 'b> {
        self.color = (red, green, blue, alpha);
        self
    }

    /// Specifies the rotation (in radians) and pivot (which is
    /// relative to the quad's `x` and `y` coordinates) of the quad.
    #[inline]
    pub fn with_rotation(
        mut self,
        rotation: f32,
        pivot_x: f32,
        pivot_y: f32,
    ) -> Renderable<'a, 'b> {
        self.rotation = (rotation, pivot_x, pivot_y);
        self
    }
}
