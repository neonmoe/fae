use crate::gl::types::*;
use crate::image::Image;
use crate::renderer::{DrawCallHandle, DrawCallParameters, Renderer, Shaders};
use crate::text::types::*;

// TODO: Gaps between glyphs or tighter uvs to avoid bleeding

const GLYPH_CACHE_WIDTH: u32 = 1024;
const GLYPH_CACHE_HEIGHT: u32 = 1024;

const DEFAULT_TEXT_SHADERS: Shaders = Shaders {
    vertex_shader_110: include_str!("../shaders/legacy/texquad.vert"),
    fragment_shader_110: include_str!("../shaders/legacy/text.frag"),
    vertex_shader_330: include_str!("../shaders/texquad.vert"),
    fragment_shader_330: include_str!("../shaders/text.frag"),
};

#[derive(Clone, Copy, PartialEq)]
pub enum ExpiryStatus {
    UsedDuringThisFrame,
    UsedDuringLastFrame,
    Expired,
}

impl ExpiryStatus {
    fn expire_one_step(&mut self) {
        use ExpiryStatus::*;
        match *self {
            UsedDuringThisFrame => *self = UsedDuringLastFrame,
            UsedDuringLastFrame => *self = Expired,
            _ => {}
        }
    }

    fn as_u32(&self) -> u32 {
        use ExpiryStatus::*;
        match *self {
            UsedDuringThisFrame => 0,
            UsedDuringLastFrame => 1,
            Expired => 2,
        }
    }

    fn is_less_expired(&self, other: ExpiryStatus) -> bool {
        self.as_u32() < other.as_u32()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct GlyphSpot {
    pub just_reserved: bool,
    pub uvs: RectUv,
    pub tex_rect: RectPx,
    content: char,
    #[allow(dead_code)]
    width: u32,
    status: ExpiryStatus,
}

struct GlyphLine {
    cache_width: u32,
    cache_height: u32,
    y: u32,
    height: u32,
    width_left: u32,
    spots: Vec<GlyphSpot>,
    status: ExpiryStatus,
}

impl GlyphLine {
    fn new(line_y: u32, line_height: u32, cache_width: u32, cache_height: u32) -> GlyphLine {
        GlyphLine {
            cache_width,
            cache_height,
            y: line_y,
            height: line_height,
            width_left: cache_width,
            spots: Vec::new(),
            status: ExpiryStatus::Expired,
        }
    }

    // TODO: read through this and ensure it works
    fn reserve(
        &mut self,
        content: char,
        width: u32,
        height: u32,
        can_evict: bool,
    ) -> Option<GlyphSpot> {
        for i in 0..self.spots.len() {
            if self.spots[i].content == content {
                self.spots[i].status = ExpiryStatus::UsedDuringThisFrame;
                self.status = ExpiryStatus::UsedDuringThisFrame;
                return Some(self.spots[i]);
            }
        }

        if self.width_left < width {
            if can_evict {
                unimplemented!("TODO: evict expired chars to make space for new ones")
            } else {
                None
            }
        } else {
            let (x, y, w, h) = (self.cache_width - self.width_left, self.y, width, height);
            let uvs = RectUv {
                x: x as f32 / self.cache_width as f32,
                y: y as f32 / self.cache_height as f32,
                w: w as f32 / self.cache_width as f32,
                h: h as f32 / self.cache_height as f32,
            };
            let tex_rect = RectPx {
                x: x as f32,
                y: y as f32,
                w: w as f32,
                h: h as f32,
            };
            let spot = GlyphSpot {
                just_reserved: false,
                content,
                tex_rect,
                width,
                uvs,
                status: ExpiryStatus::UsedDuringThisFrame,
            };
            self.spots.push(spot);

            self.status = ExpiryStatus::UsedDuringThisFrame;
            self.width_left -= width;

            let mut spot = spot.clone();
            spot.just_reserved = true;
            Some(spot)
        }
    }

    // TODO: read through this and ensure it works
    fn expire_one_step(&mut self) {
        let mut new_status = self.status;
        for spot in &mut self.spots {
            spot.status.expire_one_step();
            if spot.status.is_less_expired(new_status) {
                new_status = spot.status;
            }
        }
        self.status = new_status;
    }
}

pub struct GlyphCache {
    texture: GLuint,
    width: u32,
    height: u32,
    lines: Vec<GlyphLine>,
}

impl GlyphCache {
    pub fn create_cache_and_draw_call(renderer: &mut Renderer) -> (GlyphCache, DrawCallHandle) {
        let cache_image = Image::from_color(
            GLYPH_CACHE_WIDTH as i32,
            GLYPH_CACHE_HEIGHT as i32,
            &[0, 0, 0, 0],
        );
        let call = renderer.create_draw_call(DrawCallParameters {
            image: Some(cache_image),
            shaders: Some(DEFAULT_TEXT_SHADERS),
            alpha_blending: true,
            minification_smoothing: true,
            magnification_smoothing: true,
        });
        let cache = GlyphCache {
            texture: renderer.get_texture(&call),
            width: GLYPH_CACHE_WIDTH,
            height: GLYPH_CACHE_HEIGHT,
            lines: Vec::new(),
        };
        (cache, call)
    }

    pub(crate) fn reserve_uvs(&mut self, c: char, width: u32, height: u32) -> Option<GlyphSpot> {
        // First try to find space in the ends of existing lines
        for line in &mut self.lines {
            if let Some(spot) = line.reserve(c, width, height, false) {
                return Some(spot);
            }
        }
        // Then try adding a new line
        if let Some(new_line) = self.create_line(height) {
            if let Some(spot) = new_line.reserve(c, width, height, false) {
                return Some(spot);
            }
        }
        // Then try evicting old characters
        for line in &mut self.lines {
            if let Some(spot) = line.reserve(c, width, height, true) {
                return Some(spot);
            }
        }
        None
    }

    pub fn get_texture(&self) -> GLuint {
        self.texture
    }

    pub fn expire_one_step(&mut self) {
        for line in &mut self.lines {
            line.expire_one_step();
        }
    }

    fn create_line(&mut self, height: u32) -> Option<&mut GlyphLine> {
        let mut total_height = 0;
        for line in &self.lines {
            total_height += line.height;
        }
        if total_height + height <= self.height {
            self.lines.push(GlyphLine::new(
                total_height,
                height,
                self.width,
                self.height,
            ));
            let i = self.lines.len() - 1;
            Some(&mut self.lines[i])
        } else {
            None
        }
    }
}
