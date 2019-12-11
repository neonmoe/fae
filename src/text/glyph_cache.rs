use crate::gl;
use crate::gl::types::*;
use crate::image::Image;
use crate::renderer::{DrawCallHandle, Renderer, Shaders, TextureWrapping};
use crate::text::types::*;
use crate::types::*;

use fnv::FnvHashMap;
use std::cell::Cell;
use std::rc::{Rc, Weak};

// How far the glyphs are from each other (and from the texture's edges)
const GLYPH_CACHE_GAP: i32 = 1;

const TEXT_FRAGMENT_SHADER_110: &str = include_str!("../shaders/legacy/text.frag");
const TEXT_FRAGMENT_SHADER_330: &str = include_str!("../shaders/text.frag");

/// Contains reserved UVs in a glyph cache texture.
///
/// GlyphCache holds the reserved UVs from the glyph cache
/// texture. For performance reasons, a HashMap is used to cache
/// previously allocated UVs. This will take roughly 19 bytes per
/// cached glyph (2 bytes from the CacheIdentifier + 17 bytes from the
/// GlyphSpot).
pub(crate) struct GlyphCache {
    pub(crate) call: DrawCallHandle,
    width: i32,
    height: i32,
    max_size: i32,
    column_cursor: i32,
    columns: Vec<GlyphColumn>,
    cache: FnvHashMap<CacheIdentifier, Weak<GlyphSpot>>,
    requested_resize: Option<i32>,
}

impl GlyphCache {
    pub fn new(renderer: &mut Renderer, width: i32, height: i32, smoothed: bool) -> GlyphCache {
        let mut max_size = 0 as GLint;
        unsafe { gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_size) };

        let cache_image = Image::with_null_texture(
            (width as i32).min(max_size),
            (height as i32).min(max_size),
            gl::RED,
        );
        let call = renderer.create_draw_call(
            Some(&cache_image),
            Shaders {
                fragment_shader_110: TEXT_FRAGMENT_SHADER_110,
                fragment_shader_330: TEXT_FRAGMENT_SHADER_330,
                ..Default::default()
            },
            true,
            true,
            smoothed,
            (TextureWrapping::Clamp, TextureWrapping::Clamp),
            false,
        );
        let cache = GlyphCache {
            call,
            width,
            height,
            max_size,
            column_cursor: GLYPH_CACHE_GAP,
            columns: Vec::new(),
            cache: FnvHashMap::default(),
            requested_resize: None,
        };
        cache
    }

    pub fn resize_if_needed(&mut self, renderer: &mut Renderer) {
        if let Some(size) = self.requested_resize {
            self.requested_resize = None;
            renderer.resize_texture(&self.call, size, size);
            self.width = size;
            self.height = size;
            for col in &mut self.columns {
                col.height = self.height - GLYPH_CACHE_GAP * 2;
            }
        }
    }

    /// Returns a pointer to the GlyphSpot, and whether the spot was
    /// reserved just now (and requires rendering into).
    pub fn reserve(
        &mut self,
        id: CacheIdentifier,
        width: i32,
        height: i32,
    ) -> Result<(RectPx, bool), GlyphRenderingError> {
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
                    // First, try to reserve without evicting anything,
                    let reserve = |col: &mut GlyphColumn| col.reserve(width, height, false, false);
                    self.columns.iter_mut().find_map(reserve)
                })
                .or_else(|| {
                    // Then try creating a new column, and then reserve from that,
                    self.create_column(width, height)
                        .and_then(|col| col.reserve(width, height, false, false))
                })
                .or_else(|| {
                    // Then try evicting spots to make space for this glyph,
                    let reserve = |col: &mut GlyphColumn| col.reserve(width, height, true, false);
                    self.columns.iter_mut().find_map(reserve)
                })
                .or_else(|| {
                    // Then try evicting whole lines to make space for this glyph,
                    let reserve = |col: &mut GlyphColumn| col.reserve(width, height, true, true);
                    self.columns.iter_mut().find_map(reserve)
                })
                .or_else(|| {
                    // And finally, request a resize if it would be
                    // enough to fit the glyph, and then give
                    // up. Better luck next frame! (Then there should
                    // be enough space.)

                    // Note: we do not resize here, because resizing
                    // during a frame will invalidate all the
                    // previously reserved glyphs' texture coordinates
                    // as the size of the texture will change, and
                    // their UVs are calculated eagerly.

                    let new_size = if width > height {
                        (((self.column_cursor + width) as u32).next_power_of_two() as i32)
                            .min(self.max_size)
                    } else {
                        (((self.height + height) as u32).next_power_of_two() as i32)
                            .min(self.max_size)
                    };

                    if self.column_cursor + width + GLYPH_CACHE_GAP < new_size
                        || self.height + height + GLYPH_CACHE_GAP * 2 < new_size
                    {
                        self.requested_resize = Some(new_size);
                    }
                    None
                })
            {
                self.cache.insert(id, Rc::downgrade(&uvs));
                crate::profiler::write(|p| p.glyphs_rasterized += 1);
                Ok((uvs.texcoords, true))
            } else {
                Err(GlyphRenderingError::GlyphCacheFull)
            }
        }
    }

    pub fn upload_glyph<F: Fn(i32, i32) -> u8>(
        &mut self,
        renderer: &Renderer,
        spot: RectPx,
        get_color: F,
    ) {
        let tex_x = spot.x - 1;
        let tex_y = spot.y - 1;
        let width = spot.width + 2;
        let height = spot.height + 2;
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                    pixels.push(0u8);
                } else {
                    pixels.push(get_color(x - 1, y - 1));
                }
            }
        }

        let image = Image {
            pixels,
            width,
            height,
            pixel_type: gl::UNSIGNED_BYTE,
            format: gl::RED,
            null_data: false,
        };
        renderer.upload_texture_region(&self.call, (tex_x, tex_y, width, height).into(), &image);
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
        if height > self.height - GLYPH_CACHE_GAP * 2 {
            None
        } else {
            let col_width = (((width * 4).max(128) as u32).next_power_of_two() as i32)
                .min(self.width - self.column_cursor - GLYPH_CACHE_GAP);
            if col_width >= width {
                let column = GlyphColumn::new(
                    self.column_cursor,
                    GLYPH_CACHE_GAP,
                    col_width,
                    self.height - GLYPH_CACHE_GAP * 2,
                );
                self.columns.push(column);
                self.column_cursor += col_width + GLYPH_CACHE_GAP;
                let i = self.columns.len() - 1;
                Some(&mut self.columns[i])
            } else {
                None
            }
        }
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
        if width > self.width || height > self.height - GLYPH_CACHE_GAP {
            return None;
        }

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
        } else if let Some(spot) = self
            .reserve_line(height)
            .and_then(|new_line| new_line.reserve(width, height, false))
            .and_then(|spot_weak| spot_weak.upgrade())
        {
            return Some(spot);
        }

        None
    }

    // This is analogous to GlyphLine::evict_width, but for lines.
    #[allow(clippy::len_zero)]
    fn evict_line(&mut self, height: i32) -> Option<&mut GlyphLine> {
        let mut top = 0;
        let mut bottom = 0;
        let mut best_range = None;
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

            'inner: while bottom - top >= height {
                let evicted_glyphs: usize = self.lines[collected_range.clone()]
                    .iter()
                    .map(|line| line.reserved.len())
                    .sum();
                if let Some((_, _, other_evicted_glyphs)) = best_range {
                    if evicted_glyphs < other_evicted_glyphs {
                        best_range = Some((collected_range.clone(), top, evicted_glyphs))
                    }
                } else {
                    best_range = Some((collected_range.clone(), top, evicted_glyphs))
                }
                if collected_range.start < collected_range.end {
                    collected_range.start += 1;
                    let new_top_line = &self.lines[collected_range.start - 1];
                    top = new_top_line.y + new_top_line.height + GLYPH_CACHE_GAP;
                } else {
                    break 'inner;
                }
            }
        }

        if let Some((range, top, _)) = best_range {
            self.lines.splice(range, None);
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
        } else if cfg!(debug_assertions) {
            panic!("reserved a glyph in a spot that was already taken");
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

pub struct GlyphSpot {
    pub texcoords: RectPx,
    status: Cell<ExpiryStatus>,
}

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
