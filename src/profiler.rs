//! Module for reading the profiling data that `fae` collects. If the
//! `profiler` feature is disabled, all the functions are no-ops, and
//! there will be no profiling overhead. Probably not very useful
//! outside of this crate's development.

/// The data collected by `fae` internals. Returned by
/// [`profiler::read`](fn.read.html).
#[derive(Clone, Debug)]
pub struct ProfilingData {
    /// The amount of glyphs that had to be rasterized during this
    /// frame (ie. they weren't in the glyph cache, and drawing the
    /// glyph was slow, as it had to be rasterized).
    pub glyph_cache_misses: u32,
    /// The amount of glyphs that didn't have to rasterized during
    /// this frame (ie. they were in the glyph cache, and drawing the
    /// glyph was very fast).
    pub glyph_cache_hits: u32,
    /// The amount of glyphs that were rendered during this frame.
    pub glyphs_drawn: u32,
    /// The amount of quads that were rendered during this frame
    /// (includes the glyphs, each one is a quad).
    pub quads_drawn: u32,
    /// The amount of times a glyph had to be rasterized in the
    /// application so far.
    pub glyphs_rasterized: u32,
}

impl ProfilingData {
    const fn cleared() -> ProfilingData {
        ProfilingData {
            glyph_cache_misses: 0,
            glyph_cache_hits: 0,
            glyphs_drawn: 0,
            quads_drawn: 0,
            glyphs_rasterized: 0,
        }
    }
}

#[cfg(feature = "profiler")]
pub use actual::*;
#[cfg(not(feature = "profiler"))]
pub use dummy::*;

#[cfg(not(feature = "profiler"))]
mod dummy {
    use super::ProfilingData;

    static CLEARED_DATA: ProfilingData = ProfilingData::cleared();

    pub(crate) fn refresh() {}
    pub(crate) fn write<F: FnOnce(&mut ProfilingData) + Copy>(_f: F) {}

    /// Returns a copy of the last frame's profiling data. If the
    /// `profiler` feature is disabled, it will always be zeroed.
    pub fn read() -> ProfilingData {
        CLEARED_DATA.clone()
    }
}

#[cfg(feature = "profiler")]
mod actual {
    use super::ProfilingData;
    use std::sync::Mutex;

    impl ProfilingData {
        fn copy_from(&mut self, other: &ProfilingData) {
            *self = other.clone();
        }

        fn clear(&mut self) {
            *self = ProfilingData::cleared();
        }
    }

    lazy_static::lazy_static! {
        static ref FRONT: Mutex<ProfilingData> = Mutex::new(ProfilingData::cleared());
        static ref BACK: Mutex<ProfilingData> = Mutex::new(ProfilingData::cleared());
    }

    pub(crate) fn refresh() {
        if let (Ok(ref mut front), Ok(ref mut back)) = (FRONT.lock(), BACK.lock()) {
            let temp = back.glyphs_rasterized;
            front.copy_from(back);
            back.clear();
            back.glyphs_rasterized = temp;
        }
    }

    pub(crate) fn write<F: FnOnce(&mut ProfilingData) + Copy>(f: F) {
        if let Ok(ref mut instance) = BACK.lock() {
            f(instance);
        }
    }

    /// Returns a copy of the last frame's profiling data. If the
    /// `profiler` feature is disabled, it will always be zeroed and
    /// initialized on the spot.
    pub fn read() -> ProfilingData {
        if let Ok(instance) = FRONT.lock() {
            instance.clone()
        } else {
            ProfilingData::cleared()
        }
    }
}
