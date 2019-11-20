// TODO: Line eviction to allow font size changes

use crate::error::GlyphNotRenderedError;
use crate::gl::types::*;
use crate::image::Image;
use crate::renderer::{DrawCallHandle, DrawCallParameters, Renderer, Shaders, TextureWrapping};
use crate::text::types::*;
use crate::types::*;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

// How far the glyphs are from the texture's edges
const GLYPH_CACHE_MARGIN: i32 = 1;
// How far the glyphs are from each other
const GLYPH_CACHE_GAP: i32 = 1;

const TEXT_FRAGMENT_SHADER_110: &str = include_str!("../shaders/legacy/text.frag");
const TEXT_FRAGMENT_SHADER_330: &str = include_str!("../shaders/text.frag");

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ExpiryStatus {
    UsedDuringThisFrame,
    UsedDuringLastFrame,
    Expired,
}

impl ExpiryStatus {
    fn one_step_expired(self) -> Self {
        use ExpiryStatus::*;
        match self {
            UsedDuringThisFrame => UsedDuringLastFrame,
            UsedDuringLastFrame => Expired,
            _ => self,
        }
    }
}

pub struct GlyphSpot {
    pub texcoords: RectPx,
    status: Cell<ExpiryStatus>,
}

struct GlyphLine {
    cache_texture: GLuint,
    cache_width: i32,
    y: i32,
    height: i32,
    min_height: i32,
    max_height: i32,
    reserved: Vec<Rc<GlyphSpot>>,
}

impl GlyphLine {
    fn new(
        line_y: i32,
        line_height: i32,
        cache_texture: GLuint,
        cache_width: i32,
        cache_height: i32,
    ) -> GlyphLine {
        GlyphLine {
            cache_texture,
            cache_width,
            y: line_y,
            height: line_height,
            min_height: line_height / 2,
            max_height: (line_height * 2).min(cache_height - GLYPH_CACHE_MARGIN - line_y),
            reserved: Vec::new(),
        }
    }

    /// Iterates through this line's reserved spots, and if enough
    /// evictable\* spots (whose removal would result in a
    /// `width`-wide empty space) are found, those spots will be
    /// cleared, and the leftmost x-coordinate\*\* of this empty space
    /// will be returned.
    ///
    /// \* Spots which have not been touched since the last frame,
    ///    ie. have not been used in the draw call that is currently
    ///    being composed.
    ///
    /// \*\* Which will be GLYPH_CACHE_GAP away from the empty space's
    ///      left neighbor, or GLYPH_CACHE_MARGIN away from the cache
    ///      texture's left side, if there is no neighbor.
    #[allow(clippy::len_zero)]
    fn evict_width(&mut self, width: i32) -> Option<i32> {
        let mut left = 0;
        let mut right = 0;
        let mut collected_range = 0..0;
        for i in 0..self.reserved.len() {
            let spot = &self.reserved[i];

            if spot.status.get() == ExpiryStatus::UsedDuringThisFrame {
                collected_range = 0..0;
            } else {
                if collected_range.len() == 0 {
                    left = if i > 0 {
                        let previous = self.reserved[i - 1].texcoords;
                        previous.x + previous.width + GLYPH_CACHE_GAP
                    } else {
                        GLYPH_CACHE_MARGIN
                    };
                    collected_range = i..i;
                }
                right = if i + 1 < self.reserved.len() {
                    self.reserved[i + 1].texcoords.x - GLYPH_CACHE_GAP
                } else {
                    self.cache_width - GLYPH_CACHE_MARGIN
                };
                collected_range.end += 1;
            }

            if right - left >= width {
                break;
            }
        }

        if right - left >= width {
            let texture = self.cache_texture;
            for spot in self.reserved.splice(collected_range, None) {
                clear_texture_area(texture, spot.texcoords);
            }
            Some(left)
        } else {
            None
        }
    }

    /// Iterates through this line's spots, and tries to find a
    /// `width`-wide gap in between them\*. If such a gap is found,
    /// the leftmost x-coordinate\*\* of the gap is returned.
    ///
    /// \* With GLYPH_CACHE_GAP and GLYPH_CACHE_MARGIN taken into
    ///    account.
    /// \*\* Which, to be clear, is GLYPH_CACHE_GAP away from its
    ///      left neighbor, or GLYPH_CACHE_MARGIN away from the cache
    ///      texture's left side, if there is no left neighbor.
    fn reserve_width(&mut self, width: i32) -> Option<i32> {
        if self.reserved.is_empty() {
            return Some(GLYPH_CACHE_MARGIN);
        }

        // Compare distances between the left border of each spot and
        // the right border of the previous one.
        for i in 0..=self.reserved.len() {
            let previous_spot_right = if i > 0 {
                let previous = self.reserved[i - 1].texcoords;
                previous.x + previous.width + GLYPH_CACHE_GAP
            } else {
                GLYPH_CACHE_MARGIN
            };
            let current_spot_left = if i < self.reserved.len() {
                self.reserved[i].texcoords.x - GLYPH_CACHE_GAP
            } else {
                self.cache_width - GLYPH_CACHE_MARGIN
            };

            if current_spot_left >= previous_spot_right + width {
                return Some(previous_spot_right);
            }
        }

        None
    }

    fn reserve(&mut self, width: i32, height: i32, can_evict: bool) -> Option<Weak<GlyphSpot>> {
        if height < self.min_height || height > self.max_height {
            return None;
        }

        let x = if can_evict {
            self.evict_width(width)?
        } else {
            self.reserve_width(width)?
        };

        let spot = Rc::new(GlyphSpot {
            texcoords: RectPx {
                x,
                y: self.y,
                width,
                height,
            },
            status: Cell::new(ExpiryStatus::UsedDuringThisFrame),
        });
        let spot_weak = Rc::downgrade(&spot);

        if let Err(i) = self
            .reserved
            .binary_search_by(|elem| elem.texcoords.x.cmp(&x))
        {
            self.reserved.splice(i..i, Some(spot));
        } else {
            log::warn!("reserved a glyph in a spot that was already taken");
        }
        self.height = self.height.max(height);

        Some(spot_weak)
    }

    fn expire_one_step(&mut self) {
        for spot in &mut self.reserved {
            spot.status.set(spot.status.get().one_step_expired());
        }
    }
}

pub struct GlyphCache {
    texture: GLuint,
    width: i32,
    height: i32,
    lines: Vec<GlyphLine>,
    cache: HashMap<CacheIdentifier, Weak<GlyphSpot>>,
}

impl GlyphCache {
    pub fn create_cache_and_draw_call(
        renderer: &mut Renderer,
        width: i32,
        height: i32,
        smoothed: bool,
    ) -> (GlyphCache, DrawCallHandle) {
        let cache_image =
            Image::from_color(width as i32, height as i32, &[0]).with_format(crate::gl::RED);
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
            srgb: false,
        });
        let cache = GlyphCache {
            texture: renderer.get_texture(&call),
            width,
            height,
            lines: Vec::new(),
            cache: HashMap::new(),
        };
        (cache, call)
    }

    /// Returns a pointer to the GlyphSpot, and whether the spot was
    /// reserved just now (and requires rendering into).
    pub(crate) fn reserve_uvs(
        &mut self,
        id: CacheIdentifier,
        width: i32,
        height: i32,
    ) -> Result<(RectPx, bool), GlyphNotRenderedError> {
        if let Some(uvs) = self
            // First try to find the uvs from the cache
            .get_uvs_from_cache(id)
        {
            Ok((uvs.texcoords, false))
        } else if let Some(uvs) = self
            // Then try to find space in the existing lines
            .reserve_uvs_from_existing(width, height, false)
            // Then try adding a new line
            .or_else(|| self.reserve_uvs_from_new_line(width, height))
            // Then try evicting old characters
            .or_else(|| self.reserve_uvs_from_existing(width, height, true))
        // TODO: Add glyph cache texture re-sizing as a last resort
        {
            self.cache.insert(id, Rc::downgrade(&uvs));
            Ok((uvs.texcoords, true))
        } else {
            Err(GlyphNotRenderedError::GlyphCacheFull)
        }
    }

    fn get_uvs_from_cache(&mut self, id: CacheIdentifier) -> Option<Rc<GlyphSpot>> {
        if let Some(spot) = self.cache.get(&id) {
            if let Some(spot) = spot.upgrade() {
                spot.status.set(ExpiryStatus::UsedDuringThisFrame);
                return Some(spot);
            } else {
                self.cache.remove(&id);
            }
        }
        None
    }

    fn reserve_uvs_from_existing(
        &mut self,
        width: i32,
        height: i32,
        can_evict: bool,
    ) -> Option<Rc<GlyphSpot>> {
        for line in &mut self.lines {
            if let Some(spot) = line
                .reserve(width, height, can_evict)
                .and_then(|spot_weak| spot_weak.upgrade())
            {
                return Some(spot);
            }
        }
        None
    }

    fn reserve_uvs_from_new_line(&mut self, width: i32, height: i32) -> Option<Rc<GlyphSpot>> {
        if let Some(spot) = self
            .create_line(height)
            .and_then(|new_line| new_line.reserve(width, height, false))
            .and_then(|spot_weak| spot_weak.upgrade())
        {
            return Some(spot);
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
        if total_height + height <= self.height - GLYPH_CACHE_MARGIN {
            if !self.lines.is_empty() {
                let i = self.lines.len() - 1;
                self.lines[i].max_height = self.lines[i].height;
            }
            self.lines.push(GlyphLine::new(
                total_height,
                height,
                self.texture,
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

fn clear_texture_area(texture: GLuint, texcoords: RectPx) {
    let data = vec![0; (texcoords.width * texcoords.height) as usize];
    unsafe {
        use crate::gl;
        use crate::gl::types::*;
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexSubImage2D(
            gl::TEXTURE_2D,            // target
            0,                         // level
            texcoords.x,               // xoffset
            texcoords.y,               // yoffset
            texcoords.width,           // width
            texcoords.height,          // height
            gl::RED as GLuint,         // format
            gl::UNSIGNED_BYTE,         // type
            data.as_ptr() as *const _, // pixels
        );
    }
}
