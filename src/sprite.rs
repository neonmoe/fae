use crate::renderer::{DrawCallHandle, Renderer};
use crate::types::*;

// TODO(0.5.1): Add an anchoring system for sprites for smooth resizes.
// - The simpler way, `with_anchor(x, y)`, would anchor the whole
//   sprite to some corner.
// - The more advanced version would involve specifying the
//   top-left/bottom-right corners and their individual anchors.
// - The anchors should be specified in 0..1 floats, describing the %
//   of the way between the left/right and top/bottom edges of the
//   window. See also: how Unity does its GUIs.

/// Sprite builder struct. Call [`finish`](struct.Sprite.html#method.finish) to draw the sprite.
///
/// Created by [`DrawCallHandle::draw`](struct.DrawCallHandle.html#method.draw).
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
    pub(crate) fn create(renderer: &'a mut Renderer, call: &'b DrawCallHandle) -> Sprite<'a, 'b> {
        Sprite {
            renderer,
            call,
            z: 0.0,
            coords: (0.0, 0.0, 0.0, 0.0),
            texcoords: (-1.0, -1.0, -1.0, -1.0),
            color: (1.0, 1.0, 1.0, 1.0),
            rotation: (0.0, 0.0, 0.0),
            clip_area: None,
        }
    }

    /// Renders the quad specified by this struct using the given draw
    /// call and z coordinate. Smaller Z-values are rendered on top.
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

    /// Specifies the Z-coordinate of the sprite. Sprites with a
    /// higher Z-coordinate will be rendered over ones with a lower
    /// Z-coordinate.
    ///
    /// ## Alpha blending and Z-coordinates
    ///
    /// When drawing sprites on top of each other, with
    /// [`alpha_blending`][alpha_blending] set to true, draw the ones
    /// with the highest Z-coordinate the last, and avoid overlapping
    /// the minimum and maximum Z-coordinate ranges between draw
    /// calls.
    ///
    /// Explanation: Draw call rendering order is decided by the
    /// highest z-coordinate that each call has to draw. To get proper
    /// blending, the sprites furthest back need to be rendered
    /// first. Therefore, if a draw call is ordered to be rendered the
    /// last, but has sprites behind some other sprites, they will not
    /// get blended as hoped. However, this ordering only applies
    /// between draw calls that have
    /// [`alpha_blending`][alpha_blending]
    /// set to `true`: non-blended draw calls are always drawn before
    /// blended ones.
    ///
    /// [alpha_blending]: struct.DrawCallParameters.html#structfield.alpha_blending
    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// Specifies the screen coordinates (in logical pixels) where the
    /// quad is drawn.
    pub fn with_coordinates<R: Into<Rect>>(mut self, rect: R) -> Self {
        self.coords = rect.into().into_corners();
        self
    }

    /// Specifies the screen coordinates (in *physical* pixels) where
    /// the quad is drawn.
    pub fn with_physical_coordinates<R: Into<Rect>>(mut self, rect: R) -> Self {
        let (x0, y0, x1, y1) = rect.into().into_corners();
        let df = self.renderer.dpi_factor;
        self.coords = (x0 / df, y0 / df, x1 / df, y1 / df);
        self
    }

    /// Specifies the texture coordinates (in actual pixels, in the
    /// texture's coordinate space) from where the quad is sampled.
    pub fn with_texture_coordinates<R: Into<Rect>>(mut self, rect: R) -> Self {
        let (tw, th) = self.renderer.get_texture_size(self.call);
        let (tw, th) = (tw as f32, th as f32);
        let (x0, y0, x1, y1) = rect.into().into_corners();
        self.texcoords = (x0 / tw, y0 / th, x1 / tw, y1 / th);
        self
    }

    /// Rounds previously set coordinates
    /// ([`with_coordinates`](#method.with_coordinates)) so that they
    /// align with the physical pixels of the monitor.
    ///
    /// This might help you with weird visual glitches, especially if
    /// you're trying to render quads that have the same physical
    /// pixel size as the texture it's sampling.
    pub fn with_pixel_alignment(mut self) -> Self {
        let (x0, y0, x1, y1) = self.coords;
        let dpi_factor = self.renderer.dpi_factor;
        let round_px = |x: f32| (x * dpi_factor).round() / dpi_factor;
        let (w, h) = (round_px(x1 - x0), round_px(y1 - y0));
        let (x0, y0) = (round_px(x0), round_px(y0));
        let (x1, y1) = (x0 + w, y0 + h);
        self.coords = (x0, y0, x1, y1);
        self
    }

    /// Specifies the texture coordinates (as UVs, ie. 0.0 - 1.0) from
    /// where the quad is sampled.
    pub fn with_uvs<R: Into<Rect>>(mut self, rect: R) -> Self {
        self.texcoords = rect.into().into_corners();
        self
    }

    /// Specifies the clip area. Only the parts that overlap between
    /// the clip area and the area specified by
    /// [`with_coordinates`](#method.with_coordinates) are rendered.
    pub fn with_clip_area<R: Into<Rect>>(mut self, rect: R) -> Self {
        self.clip_area = Some(rect.into().into_corners());
        self
    }

    /// Specifies the color tint of the quad.
    pub fn with_color(mut self, (red, green, blue, alpha): (f32, f32, f32, f32)) -> Self {
        self.color = (red, green, blue, alpha);
        self
    }

    /// Specifies the rotation (in radians) and pivot of the quad,
    /// relative to the sprite's origin.
    pub fn with_rotation(mut self, rotation: f32, pivot_x: f32, pivot_y: f32) -> Self {
        self.rotation = (rotation, pivot_x, pivot_y);
        self
    }
}
