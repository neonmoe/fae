use crate::text::{Alignment, TextRenderer};
use crate::types::*;

/// Text builder struct. Call
/// [`finish`](struct.Text.html#method.finish) to draw the text.
///
/// Created by
/// [`TextRenderer::draw`](struct.TextRenderer.html#method.draw).
pub struct Text<'a> {
    renderer: &'a mut TextRenderer,
    inner: TextCacheable,
    z: f32,
    clip_area: Option<Rect>,
    color: (f32, f32, f32, f32),
    rotation: (f32, f32, f32),
    visible: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct TextCacheable {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub font_size: i32,
    pub alignment: Alignment,
    pub max_line_width: Option<i32>,
}

impl<'a> Text<'a> {
    pub(crate) fn new(
        renderer: &'a mut TextRenderer,
        text: String,
        x: i32,
        y: i32,
        z: f32,
        font_size: i32,
    ) -> Text<'a> {
        Text {
            renderer,
            inner: TextCacheable {
                text,
                x,
                y,
                font_size,
                alignment: Alignment::Left,
                max_line_width: None,
            },
            z,
            clip_area: None,
            color: (0.0, 0.0, 0.0, 1.0),
            rotation: (0.0, 0.0, 0.0),
            visible: true,
        }
    }

    /// Draws the text, and returns the bounding box of all the glyphs
    /// drawn, if any were.
    pub fn finish(self) -> Option<Rect> {
        self.renderer.draw_text(
            self.inner,
            self.z,
            self.clip_area,
            self.color,
            self.rotation,
            self.visible,
        )
    }

    /// Sets the text's color.
    pub fn with_color(mut self, (red, green, blue, alpha): (f32, f32, f32, f32)) -> Self {
        self.color = (red, green, blue, alpha);
        self
    }

    /// Sets the text's alignment. Does nothing if max width isn't
    /// specified: the x-coordinate of this text is considered to be
    /// the left border, and the max width of the text + the
    /// x-coordinate the right.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.inner.alignment = alignment;
        self
    }

    /// Sets the maximum width of this text.
    pub fn with_max_width(mut self, width: f32) -> Self {
        self.inner.max_line_width = Some((width * self.renderer.dpi_factor) as i32);
        self
    }

    /// Sets the clipping area, ie. the area where the text will be
    /// rendered. Text that falls out of the clip area will be
    /// *clipped* off.
    pub fn with_clip_area<R: Into<Rect>>(mut self, clip_area: R) -> Self {
        self.clip_area = Some(clip_area.into());
        self
    }

    /// Sets the visibility of the text. If false, the text will not
    /// be rendered. That is, only the bounding box of the text will
    /// be calculated.
    ///
    /// Useful for measuring the bounding box of some piece of text,
    /// without spending performance drawing it with 0 alpha, which
    /// would make it invisible as well.
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Specifies the rotation (in radians) and pivot\* of the text.
    ///
    /// \* Relative to the text's `x` and `y` parameters.
    pub fn with_rotation(mut self, rotation: f32, pivot_x: f32, pivot_y: f32) -> Self {
        self.rotation = (rotation, pivot_x, pivot_y);
        self
    }
}
