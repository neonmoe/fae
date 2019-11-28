use crate::text::{Alignment, TextRenderer};
use crate::types::*;

/// Text builder struct. Call
/// [`finish`](struct.Text.html#method.finish) to draw the text.
///
/// Created by
/// [`FontHandle::draw`](struct.FontHandle.html#method.draw).
pub struct Text<'a> {
    renderer: &'a mut TextRenderer,
    data: TextData,
}

pub(crate) struct TextData {
    pub x: i32,
    pub y: i32,
    pub z: f32,
    pub font_size: i32,
    pub text: String,
    pub alignment: Alignment,
    pub max_line_width: Option<i32>,
    pub color: (f32, f32, f32, f32),
    pub rotation: (f32, f32, f32),
    pub clip_area: Option<Rect>,
    pub visible: bool,
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
            data: TextData {
                text,
                x,
                y,
                z,
                font_size,
                alignment: Alignment::Left,
                max_line_width: None,
                clip_area: None,
                color: (0.0, 0.0, 0.0, 1.0),
                rotation: (0.0, 0.0, 0.0),
                visible: true,
            },
        }
    }

    /// Draws the text, and returns the bounding box of all the glyphs
    /// drawn, if any were.
    pub fn finish(self) -> Option<Rect> {
        self.renderer.draw_text(self.data)
    }

    /// Sets the text's color.
    pub fn with_color(mut self, (red, green, blue, alpha): (f32, f32, f32, f32)) -> Self {
        self.data.color = (red, green, blue, alpha);
        self
    }

    /// Sets the text's alignment. Does nothing if max width isn't
    /// specified: the x-coordinate of this text is considered to be
    /// the left border, and the max width of the text + the
    /// x-coordinate the right.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.data.alignment = alignment;
        self
    }

    /// Sets the maximum width of this text.
    pub fn with_max_width(mut self, width: f32) -> Self {
        self.data.max_line_width = Some((width * self.renderer.dpi_factor) as i32);
        self
    }

    /// Sets the clipping area, ie. the area where the text will be
    /// rendered. Text that falls out of the clip area will be
    /// *clipped* off.
    pub fn with_clip_area<R: Into<Rect>>(mut self, clip_area: R) -> Self {
        self.data.clip_area = Some(clip_area.into());
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
        self.data.visible = visible;
        self
    }

    /// Specifies the rotation (in radians) and pivot of the text,
    /// relative to the text's origin.
    pub fn with_rotation(mut self, rotation: f32, pivot_x: f32, pivot_y: f32) -> Self {
        self.data.rotation = (rotation, pivot_x, pivot_y);
        self
    }
}
