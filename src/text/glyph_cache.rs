// TODO: Line eviction to allow font size changes
// Current idea:
// - Divide the texture into n * 128 wide sections
//   (n should be the minimum needed to fit a requested character)
// - Lines work as now, but as a last-resort before resizing the texture,
//   they should be gone through much like GlyphLine's eviction, but vertically.
//   (it will probably need to loop through all glyphs to check for line expiry, but it's
//   still better than resizing to conserve VRAM)

use crate::error::GlyphNotRenderedError;
use crate::gl::types::*;
use crate::image::Image;
use crate::renderer::{DrawCallHandle, DrawCallParameters, Renderer, Shaders, TextureWrapping};
use crate::text::types::*;
use crate::types::*;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

// How far the glyphs are from each other (and from the texture's edges)
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
    texture: GLuint,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    min_height: i32,
    max_height: i32,
    reserved: Vec<Rc<GlyphSpot>>,
}

impl GlyphLine {
    fn new(
        texture: GLuint,
        cache_height: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> GlyphLine {
        GlyphLine {
            texture,
            x,
            y,
            width,
            height,
            min_height: height / 2,
            max_height: (height * 2).min(cache_height - GLYPH_CACHE_GAP - y),
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
    ///      left neighbor.
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
                        self.x
                    };
                    collected_range = i..i;
                }
                right = if i + 1 < self.reserved.len() {
                    self.reserved[i + 1].texcoords.x - GLYPH_CACHE_GAP
                } else {
                    self.x + self.width
                };
                collected_range.end += 1;
            }

            if right - left >= width {
                break;
            }
        }

        if right - left >= width {
            let texture = self.texture;
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
    /// \* With GLYPH_CACHE_GAP taken into account.
    /// \*\* Which, to be clear, is GLYPH_CACHE_GAP away from its
    ///      left neighbor.
    fn reserve_width(&mut self, width: i32) -> Option<i32> {
        if self.reserved.is_empty() {
            return Some(self.x);
        }

        // Compare distances between the left border of each spot and
        // the right border of the previous one.
        for i in 0..=self.reserved.len() {
            let previous_spot_right = if i > 0 {
                let previous = self.reserved[i - 1].texcoords;
                previous.x + previous.width + GLYPH_CACHE_GAP
            } else {
                self.x
            };
            let current_spot_left = if i < self.reserved.len() {
                self.reserved[i].texcoords.x - GLYPH_CACHE_GAP
            } else {
                self.x + self.width
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
}

struct GlyphColumn {
    texture: GLuint,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    lines: Vec<GlyphLine>,
}

impl GlyphColumn {
    fn new(texture: GLuint, x: i32, y: i32, width: i32, height: i32) -> GlyphColumn {
        GlyphColumn {
            texture,
            x,
            y,
            width,
            height,
            lines: Vec::new(),
        }
    }

    fn reserve_uvs(&mut self, width: i32, height: i32, can_evict: bool) -> Option<Rc<GlyphSpot>> {
        // First try finding space in existing lines
        for line in &mut self.lines {
            if let Some(spot) = line
                .reserve(width, height, can_evict)
                .and_then(|spot_weak| spot_weak.upgrade())
            {
                return Some(spot);
            }
        }

        // Then try creating a new one
        if let Some(spot) = self
            .create_line(height)
            .and_then(|new_line| new_line.reserve(width, height, false))
            .and_then(|spot_weak| spot_weak.upgrade())
        {
            return Some(spot);
        }

        None
    }

    fn create_line(&mut self, height: i32) -> Option<&mut GlyphLine> {
        let mut y = self.y;
        for line in &self.lines {
            y += line.height + GLYPH_CACHE_GAP;
        }
        if y + height <= self.y + self.height {
            if !self.lines.is_empty() {
                let i = self.lines.len() - 1;
                self.lines[i].max_height = self.lines[i].height;
            }
            self.lines.push(GlyphLine::new(
                self.texture,
                self.height,
                self.x,
                y,
                self.width,
                height,
            ));
            let i = self.lines.len() - 1;
            Some(&mut self.lines[i])
        } else {
            None
        }
    }
}

pub struct GlyphCache {
    texture: GLuint,
    width: i32,
    height: i32,
    column_cursor: i32,
    columns: Vec<GlyphColumn>,
    cache: HashMap<CacheIdentifier, Weak<GlyphSpot>>,
}

impl GlyphCache {
    pub fn create_cache_and_draw_call(
        renderer: &mut Renderer,
        width: i32,
        height: i32,
        smoothed: bool,
    ) -> (GlyphCache, DrawCallHandle) {
        let cache_image = Image::create_null(width as i32, height as i32, crate::gl::RED);
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
            column_cursor: GLYPH_CACHE_GAP,
            columns: Vec::new(),
            cache: HashMap::new(),
        };
        (cache, call)
    }

    /// Returns a pointer to the GlyphSpot, and whether the spot was
    /// reserved just now (and requires rendering into).
    pub fn reserve_uvs(
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
        } else {
            // Crawl through the columns trying different ways to
            // reserve a spot, each more invasive than the last.
            // Start with None to make the chain more uniform.
            if let Some(uvs) = None
                .or_else(|| {
                    let reserve = |col: &mut GlyphColumn| col.reserve_uvs(width, height, false);
                    self.columns.iter_mut().find_map(reserve)
                })
                .or_else(|| {
                    self.create_column(width, height)
                        .and_then(|col| col.reserve_uvs(width, height, true))
                })
                .or_else(|| {
                    let reserve = |col: &mut GlyphColumn| col.reserve_uvs(width, height, true);
                    self.columns.iter_mut().find_map(reserve)
                })
                .or_else(|| {
                    // TODO: Try resizing the texture and then add a new column to reserve from
                    // TODO: Resize column heights on resize
                    None
                })
            {
                self.cache.insert(id, Rc::downgrade(&uvs));
                return Ok((uvs.texcoords, true));
            }
            Err(GlyphNotRenderedError::GlyphCacheFull)
        }
    }

    pub fn get_texture(&self) -> GLuint {
        self.texture
    }

    pub fn expire_one_step(&mut self) {
        let mut removed_spots = Vec::new();
        for (key, spot) in self.cache.iter_mut() {
            if let Some(spot) = spot.upgrade() {
                spot.status.set(spot.status.get().one_step_expired());
            } else {
                removed_spots.push(key.clone());
            }
        }
        for spot_key in removed_spots {
            self.cache.remove(&spot_key);
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

    fn create_column(&mut self, width: i32, height: i32) -> Option<&mut GlyphColumn> {
        if height > self.height - GLYPH_CACHE_GAP - self.column_cursor {
            None
        } else {
            let mut x = GLYPH_CACHE_GAP;
            for col in &self.columns {
                x += col.width + GLYPH_CACHE_GAP;
            }
            let col_width = (((width * 4).max(128) as u32).next_power_of_two() as i32)
                .min(self.width - x - GLYPH_CACHE_GAP);
            if col_width >= width {
                let column = GlyphColumn::new(
                    self.texture,
                    x,
                    self.column_cursor,
                    col_width,
                    self.height - GLYPH_CACHE_GAP - self.column_cursor,
                );
                self.columns.push(column);
                let i = self.columns.len() - 1;
                Some(&mut self.columns[i])
            } else {
                None
            }
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
