use crate::renderer::{DrawCallHandle, Renderer};
use crate::types::*;

/// Sprite builder struct. Call [`finish()`](struct.Sprite.html#method.finish) to draw the sprite.
///
/// Created by [`Renderer::draw()`](struct.Renderer.html#method.draw).
pub struct Sprite<'a, 'b> {
    renderer: &'a mut Renderer,
    call: &'b DrawCallHandle,
    z: f32,
    coords: (f32, f32, f32, f32),
    texcoords: (f32, f32, f32, f32),
    color: (f32, f32, f32, f32),
    rotation: (f32, f32, f32),
    clip_area: Option<(f32, f32, f32, f32)>,
}

impl<'a, 'b> Sprite<'a, 'b> {
    #[inline]
    pub(crate) fn create(
        renderer: &'a mut Renderer,
        call: &'b DrawCallHandle,
        z: f32,
    ) -> Sprite<'a, 'b> {
        Sprite {
            renderer,
            call,
            z,
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
                area,
                self.coords,
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
    pub fn with_coordinates<R: Into<Rect>>(mut self, rect: R) -> Sprite<'a, 'b> {
        self.coords = rect.into().into_corners();
        self
    }

    /// Specifies the screen coordinates (in *physical* pixels) where
    /// the quad is drawn.
    #[inline]
    pub fn with_physical_coordinates<R: Into<Rect>>(mut self, rect: R) -> Sprite<'a, 'b> {
        let (x0, y0, x1, y1) = rect.into().into_corners();
        let df = self.renderer.dpi_factor;
        self.coords = (x0 / df, y0 / df, x1 / df, y1 / df);
        self
    }

    /// Specifies the texture coordinates (in actual pixels, in the
    /// texture's coordinate space) from where the quad is sampled.
    #[inline]
    pub fn with_texture_coordinates<R: Into<Rect>>(mut self, rect: R) -> Sprite<'a, 'b> {
        let (tw, th) = self.renderer.get_texture_size(self.call);
        let (tw, th) = (tw as f32, th as f32);
        let (x0, y0, x1, y1) = rect.into().into_corners();
        self.texcoords = (x0 / tw, y0 / th, x1 / tw, y1 / th);
        self
    }

    /// Rounds previously set cooordinates
    /// ([`with_coordinates`](#method.with_coordinates)) so that they
    /// align with the physical pixels of the monitor.
    ///
    /// This might help you with weird visual glitches, especially if
    /// you're trying to render quads that have the same physical
    /// pixel size as the texture it's sampling.
    #[inline]
    pub fn with_pixel_alignment(mut self) -> Sprite<'a, 'b> {
        let (x0, y0, x1, y1) = self.coords;
        let dpi_factor = self.renderer.dpi_factor;
        let round_px = |x: f32| (x * dpi_factor).round() / dpi_factor;
        let (x0, y0, x1, y1) = (round_px(x0), round_px(y0), round_px(x1), round_px(y1));
        self.coords = (x0, y0, x1, y1);
        self
    }

    /// Specifies the texture coordinates (as UVs) from where the quad is sampled.
    #[inline]
    pub fn with_uvs<R: Into<Rect>>(mut self, rect: R) -> Sprite<'a, 'b> {
        self.texcoords = rect.into().into_corners();
        self
    }

    /// Specifies the clip area. Only the parts that overlap between
    /// the clip area and this quad are rendered.
    #[inline]
    pub fn with_clip_area(mut self, rect: Rect) -> Sprite<'a, 'b> {
        self.clip_area = Some(rect.into_corners());
        self
    }

    /// Specifies the color tint of the quad.
    #[inline]
    pub fn with_color(mut self, red: f32, green: f32, blue: f32, alpha: f32) -> Sprite<'a, 'b> {
        self.color = (red, green, blue, alpha);
        self
    }

    /// Specifies the rotation (in radians) and pivot (which is
    /// relative to the quad's `x` and `y` coordinates) of the quad.
    #[inline]
    pub fn with_rotation(mut self, rotation: f32, pivot_x: f32, pivot_y: f32) -> Sprite<'a, 'b> {
        self.rotation = (rotation, pivot_x, pivot_y);
        self
    }
}