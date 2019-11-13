use crate::gl::types::*;
use crate::image::Image;
use crate::renderer::{DrawCallHandle, DrawCallParameters, Renderer, Shaders, TextureWrapping};
use crate::text::types::*;
use crate::types::*;

use std::collections::HashMap;

// How far the glyphs are from the texture's edges
const GLYPH_CACHE_MARGIN: i32 = 1;
// How far the glyphs are from each other
const GLYPH_CACHE_GAP: i32 = 1;

const TEXT_FRAGMENT_SHADER_110: &'static str = include_str!("../shaders/legacy/text.frag");
const TEXT_FRAGMENT_SHADER_330: &'static str = include_str!("../shaders/text.frag");

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
    pub texcoords: RectPx,
    #[allow(dead_code)]
    width: i32, // for cache eviction
    status: ExpiryStatus,
}

struct GlyphLine {
    cache_width: i32,
    #[allow(dead_code)]
    cache_height: i32, // TODO: Allow height stretching up to double of the smallest glyph, but don't go over this
    y: i32,
    height: i32,
    width_left: i32,
    spots: HashMap<CacheIdentifier, GlyphSpot>,
}

impl GlyphLine {
    fn new(line_y: i32, line_height: i32, cache_width: i32, cache_height: i32) -> GlyphLine {
        GlyphLine {
            cache_width,
            cache_height,
            y: line_y,
            height: line_height,
            width_left: cache_width - GLYPH_CACHE_MARGIN,
            spots: HashMap::new(),
        }
    }

    fn reserve(
        &mut self,
        id: CacheIdentifier,
        width: i32,
        height: i32,
        can_evict: bool,
    ) -> Option<GlyphSpot> {
        if let Some(spot) = self.spots.get_mut(&id) {
            spot.status = ExpiryStatus::UsedDuringThisFrame;
            return Some(*spot);
        }

        if self.width_left - GLYPH_CACHE_MARGIN <= width {
            if can_evict {
                // TODO: evict expired chars to make space for new ones
                None
            } else {
                None
            }
        } else {
            let texcoords = RectPx {
                x: self.cache_width - self.width_left + GLYPH_CACHE_MARGIN,
                y: self.y,
                width,
                height,
            };
            let spot = GlyphSpot {
                just_reserved: false,
                texcoords,
                width,
                status: ExpiryStatus::UsedDuringThisFrame,
            };
            self.spots.insert(id, spot);
            self.width_left -= width + GLYPH_CACHE_GAP;

            let mut spot = spot.clone();
            spot.just_reserved = true;
            Some(spot)
        }
    }

    fn expire_one_step(&mut self) {
        for spot in self.spots.values_mut() {
            spot.status.expire_one_step();
        }
    }
}

pub struct GlyphCache {
    texture: GLuint,
    width: i32,
    height: i32,
    lines: Vec<GlyphLine>,
    out_of_memory: bool,
}

impl GlyphCache {
    pub fn create_cache_and_draw_call(
        renderer: &mut Renderer,
        width: i32,
        height: i32,
        smoothed: bool,
    ) -> (GlyphCache, DrawCallHandle) {
        let cache_image = Image::from_color(width as i32, height as i32, &[0, 0, 0, 0]);
        let call = renderer.create_draw_call(DrawCallParameters {
            image: Some(cache_image),
            shaders: Shaders {
                fragment_shader_110: TEXT_FRAGMENT_SHADER_110,
                fragment_shader_330: TEXT_FRAGMENT_SHADER_330,
                ..Default::default()
            },
            alpha_blending: true,
            minification_smoothing: true,
            magnification_smoothing: smoothed,
            wrap: (TextureWrapping::Clamp, TextureWrapping::Clamp),
        });
        let cache = GlyphCache {
            texture: renderer.get_texture(&call),
            width,
            height,
            lines: Vec::new(),
            out_of_memory: false,
        };
        (cache, call)
    }

    pub(crate) fn reserve_uvs(
        &mut self,
        id: CacheIdentifier,
        width: i32,
        height: i32,
    ) -> Option<GlyphSpot> {
        if let Some(uvs) = self
            // First try to find space in the ends of existing lines
            .reserve_uvs_from_existing(id, width, height, false)
            // Then try adding a new line
            .or_else(|| self.reserve_uvs_from_new_line(id, width, height))
            // Then try evicting old characters
            .or_else(|| self.reserve_uvs_from_existing(id, width, height, true))
        // TODO: Add glyph cache texture re-sizing as a last resort
        {
            Some(uvs)
        } else {
            if !self.out_of_memory {
                log::warn!("glyph cache ran out of space: {:?}", id);
            }
            self.out_of_memory = true;
            None
        }
    }

    fn reserve_uvs_from_existing(
        &mut self,
        id: CacheIdentifier,
        width: i32,
        height: i32,
        can_evict: bool,
    ) -> Option<GlyphSpot> {
        for line in &mut self.lines {
            if let Some(spot) = line.reserve(id, width, height, can_evict) {
                return Some(spot);
            }
        }
        None
    }

    fn reserve_uvs_from_new_line(
        &mut self,
        id: CacheIdentifier,
        width: i32,
        height: i32,
    ) -> Option<GlyphSpot> {
        if let Some(new_line) = self.create_line(height) {
            if let Some(spot) = new_line.reserve(id, width, height, false) {
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

    fn create_line(&mut self, height: i32) -> Option<&mut GlyphLine> {
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
