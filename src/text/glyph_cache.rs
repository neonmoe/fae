use crate::error::GlyphNotRenderedError;
use crate::gl;
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
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    min_height: i32,
    max_height: i32,
    reserved: Vec<Rc<GlyphSpot>>,
}

impl GlyphLine {
    fn new(max_height_cap: i32, x: i32, y: i32, width: i32, height: i32) -> GlyphLine {
        GlyphLine {
            x,
            y,
            width,
            height,
            min_height: height / 2,
            max_height: (height * 2).min(max_height_cap),
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
    // This is analogous to GlyphColumn::evict_line, but for glyphs.
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
            self.reserved.splice(collected_range, None);
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
    // This is analogous to GlyphColumn::reserve_line, but for glyphs.
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

    fn has_been_hit_this_frame(&self) -> bool {
        for spot in &self.reserved {
            if spot.status.get() == ExpiryStatus::UsedDuringThisFrame {
                return true;
            }
        }

        false
    }
}

struct GlyphColumn {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    lines: Vec<GlyphLine>,
}

impl GlyphColumn {
    fn new(x: i32, y: i32, width: i32, height: i32) -> GlyphColumn {
        GlyphColumn {
            x,
            y,
            width,
            height,
            lines: Vec::new(),
        }
    }

    fn reserve(
        &mut self,
        width: i32,
        height: i32,
        can_evict_spots: bool,
        can_evict_lines: bool,
    ) -> Option<Rc<GlyphSpot>> {
        // First try finding space in existing lines
        for line in &mut self.lines {
            if let Some(spot) = line
                .reserve(width, height, can_evict_spots)
                .and_then(|spot_weak| spot_weak.upgrade())
            {
                return Some(spot);
            }
        }

        // Then try creating a new one
        if can_evict_lines {
            if let Some(spot) = self
                .evict_line(height)
                .and_then(|new_line| new_line.reserve(width, height, false))
                .and_then(|spot_weak| spot_weak.upgrade())
            {
                return Some(spot);
            }
        } else {
            if let Some(spot) = self
                .reserve_line(height)
                .and_then(|new_line| new_line.reserve(width, height, false))
                .and_then(|spot_weak| spot_weak.upgrade())
            {
                return Some(spot);
            }
        }

        None
    }

    // This is analogous to GlyphLine::evict_width, but for lines.
    fn evict_line(&mut self, height: i32) -> Option<&mut GlyphLine> {
        let mut top = 0;
        let mut bottom = 0;
        let mut collected_range = 0..0;
        for i in 0..self.lines.len() {
            if self.lines[i].has_been_hit_this_frame() {
                collected_range = 0..0;
            } else {
                if collected_range.len() == 0 {
                    top = if i > 0 {
                        self.lines[i - 1].y + self.lines[i - 1].height + GLYPH_CACHE_GAP
                    } else {
                        self.y + GLYPH_CACHE_GAP
                    };
                    collected_range = i..i;
                }
                bottom = if i + 1 < self.lines.len() {
                    self.lines[i + 1].y - GLYPH_CACHE_GAP
                } else {
                    self.y + self.height - GLYPH_CACHE_GAP
                };
                collected_range.end += 1;
            }

            if bottom - top >= height {
                break;
            }
        }

        if bottom - top >= height {
            self.lines.splice(collected_range, None);
            self.create_line(top, height)
        } else {
            None
        }
    }

    // This is analogous to GlyphLine::reserve_width, but for lines.
    fn reserve_line(&mut self, height: i32) -> Option<&mut GlyphLine> {
        let mut current_best_line = None;
        for i in 0..=self.lines.len() {
            let previous_line_bottom = if i > 0 {
                self.lines[i - 1].y + self.lines[i - 1].height + GLYPH_CACHE_GAP
            } else {
                self.y + GLYPH_CACHE_GAP
            };
            let current_line_top = if i < self.lines.len() {
                self.lines[i].y - GLYPH_CACHE_GAP
            } else {
                self.y + self.height - GLYPH_CACHE_GAP
            };

            let available_height = current_line_top - previous_line_bottom;
            let y = previous_line_bottom;
            if height <= available_height {
                if let Some((_, compare_height)) = current_best_line {
                    if available_height < compare_height {
                        // Try to find the minimum fitting height
                        current_best_line = Some((y, available_height));
                    }
                } else {
                    current_best_line = Some((y, available_height));
                }
            }
        }
        if let Some((y, _)) = current_best_line {
            self.create_line(y, height)
        } else {
            None
        }
    }

    fn create_line(&mut self, y: i32, height: i32) -> Option<&mut GlyphLine> {
        if let Err(i) = self.lines.binary_search_by(|elem| elem.y.cmp(&y)) {
            // Cap the max height at the border of the next line (if there is one)
            let max_height_cap = if i < self.lines.len() {
                self.lines[i].y - y - GLYPH_CACHE_GAP
            } else {
                self.height - y - GLYPH_CACHE_GAP
            };
            let line = GlyphLine::new(max_height_cap, self.x, y, self.width, height);
            self.lines.splice(i..i, Some(line));
            // Stop the line above this one expanding over this one (if there is one)
            if i > 0 {
                self.lines[i - 1].max_height = self.lines[i - 1].height;
            }
            Some(&mut self.lines[i])
        } else {
            log::warn!("tried to reserve a line that was already occupied");
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
        let cache_image = Image::create_null(width as i32, height as i32, gl::RED);
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
    pub fn reserve(
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
                    // First, try to reserve without evicting anything
                    let reserve = |col: &mut GlyphColumn| col.reserve(width, height, false, false);
                    self.columns.iter_mut().find_map(reserve)
                })
                .or_else(|| {
                    // Then try creating a new column, and then reserve from that
                    self.create_column(width, height)
                        .and_then(|col| col.reserve(width, height, false, false))
                })
                .or_else(|| {
                    // Then try evicting spots to make space for this glyph
                    let reserve = |col: &mut GlyphColumn| col.reserve(width, height, true, false);
                    self.columns.iter_mut().find_map(reserve)
                })
                .or_else(|| {
                    // Then try evicting whole lines to make space for this glyph
                    let reserve = |col: &mut GlyphColumn| col.reserve(width, height, true, true);
                    self.columns.iter_mut().find_map(reserve)
                })
                .or_else(|| {
                    // TODO: Try resizing the texture and then add a new column to reserve from
                    // TODO: Resize column heights on resize
                    None
                })
            {
                self.cache.insert(id, Rc::downgrade(&uvs));
                Ok((uvs.texcoords, true))
            } else {
                Err(GlyphNotRenderedError::GlyphCacheFull)
            }
        }
    }

    pub fn upload_glyph<F: Fn(i32, i32) -> u8>(&mut self, spot: RectPx, get_color: F) {
        let tex_x = spot.x - 1;
        let tex_y = spot.y - 1;
        let width = spot.width + 2;
        let height = spot.height + 2;
        let mut data = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                    data.push(0u8);
                } else {
                    data.push(get_color(x - 1, y - 1));
                }
            }
        }

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,            // target
                0,                         // level
                tex_x,                     // xoffset
                tex_y,                     // yoffset
                width,                     // width
                height,                    // height
                gl::RED as GLuint,         // format
                gl::UNSIGNED_BYTE,         // type
                data.as_ptr() as *const _, // pixels
            );
            crate::renderer::print_gl_errors("after upload_glyph texsubimage2d");
        }
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
                debug_assert_eq!(Rc::strong_count(&spot), 2);
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
