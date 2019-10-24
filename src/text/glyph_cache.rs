use crate::gl::types::*;
use crate::image::Image;
use crate::renderer::{DrawCallHandle, DrawCallParameters, Renderer, Shaders, TextureWrapping};
use crate::text::types::*;

// TODO: Gaps between glyphs or tighter uvs to avoid bleeding

// How far the glyphs are from the texture's edges
const GLYPH_CACHE_MARGIN: u32 = 1;
// How far the glyphs are from each other
const GLYPH_CACHE_GAP: u32 = 1;

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
}

#[derive(Clone, Copy)]
pub(crate) struct GlyphSpot {
    pub just_reserved: bool,
    pub uvs: RectUv,
    pub tex_rect: RectPx<u32>,
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
}

impl GlyphLine {
    fn new(line_y: u32, line_height: u32, cache_width: u32, cache_height: u32) -> GlyphLine {
        GlyphLine {
            cache_width,
            cache_height,
            y: line_y,
            height: line_height,
            width_left: cache_width - GLYPH_CACHE_MARGIN,
            spots: Vec::new(),
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
                return Some(self.spots[i]);
            }
        }

        if self.width_left - GLYPH_CACHE_MARGIN < width {
            if can_evict {
                // TODO: evict expired chars to make space for new ones
                None
            } else {
                None
            }
        } else {
            let (x, y, w, h) = (
                self.cache_width - self.width_left + GLYPH_CACHE_MARGIN,
                self.y,
                width,
                height,
            );
            let uvs = RectUv {
                x: (x as f32 - 0.5) / self.cache_width as f32,
                y: (y as f32 - 0.5) / self.cache_height as f32,
                w: (w as f32 + 0.5) / self.cache_width as f32,
                h: (h as f32 + 0.5) / self.cache_height as f32,
            };
            let tex_rect = RectPx {
                x: x,
                y: y,
                w: w,
                h: h,
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
            self.width_left -= width + GLYPH_CACHE_GAP;

            let mut spot = spot.clone();
            spot.just_reserved = true;
            Some(spot)
        }
    }

    fn expire_one_step(&mut self) {
        for spot in &mut self.spots {
            spot.status.expire_one_step();
        }
    }
}

pub struct GlyphCache {
    texture: GLuint,
    width: u32,
    height: u32,
    lines: Vec<GlyphLine>,
}

impl GlyphCache {
    pub fn create_cache_and_draw_call(
        renderer: &mut Renderer,
        width: u32,
        height: u32,
    ) -> (GlyphCache, DrawCallHandle) {
        let cache_image = Image::from_color(width as i32, height as i32, &[0, 0, 0, 0]);
        let call = renderer.create_draw_call(DrawCallParameters {
            image: Some(cache_image),
            shaders: Some(DEFAULT_TEXT_SHADERS),
            alpha_blending: true,
            minification_smoothing: true,
            magnification_smoothing: true,
            wrap: (TextureWrapping::Clamp, TextureWrapping::Clamp),
        });
        let cache = GlyphCache {
            texture: renderer.get_texture(&call),
            width,
            height,
            lines: Vec::new(),
        };
        (cache, call)
    }

    pub(crate) fn reserve_uvs(&mut self, c: char, width: u32, height: u32) -> Option<GlyphSpot> {
        // First try to find space in the ends of existing lines
        self.reserve_uvs_from_existing(c, width, height, false)
            // Then try adding a new line
            .or_else(|| self.reserve_uvs_from_new_line(c, width, height))
            // Then try evicting old characters
            .or_else(|| self.reserve_uvs_from_existing(c, width, height, true))
        // TODO: Add glyph cache texture re-sizing as a last resort
    }

    fn reserve_uvs_from_existing(
        &mut self,
        c: char,
        width: u32,
        height: u32,
        can_evict: bool,
    ) -> Option<GlyphSpot> {
        for line in &mut self.lines {
            if let Some(spot) = line.reserve(c, width, height, can_evict) {
                return Some(spot);
            }
        }
        None
    }

    fn reserve_uvs_from_new_line(&mut self, c: char, width: u32, height: u32) -> Option<GlyphSpot> {
        if let Some(new_line) = self.create_line(height) {
            if let Some(spot) = new_line.reserve(c, width, height, false) {
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
        let mut total_height = GLYPH_CACHE_MARGIN;
        for line in &self.lines {
            total_height += line.height + GLYPH_CACHE_GAP;
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
