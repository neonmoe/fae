<!-- TODO(0.5.0): Rewrite the readme? -->

# Fae
Fae is a simple, performant, and compatible 2D rendering crate built
on top of `glutin`, with optional text rendering functionality.

Fae's main design goals are simplicity and performance while
supporting older/low-end target platforms. The optional text-rendering
feature `text` can draw multi-line, cached text with the `font8x8` (a
font-in-a-crate consisting of very densely encoded 8x8 glyphs) and
`rusttype` (ttf rasterizer) crates. So no ligatures or other
sophisticated font rendering.

Fae supports OpenGL 2.1 and OpenGL ES 2.0 contexts, but will do some
optimizations (VAOs, instanced rendering) if a newer context
(3.3+/ES 3.0+) is available.

This is not a serious contender intended to replace any general 2D
rendering crates. I'm developing this as an exercise to learn about
OpenGL and text rendering, with an eventual goal of being usable for
my own applications. Use with caution!

## Important note
Fae is currently under development, and I wouldn't recommend it for
any kind of usage yet. It's on crates.io mostly so I don't have to
come up with another name :)

## Cargo features
- The **png** feature implements the `Image::from_png` function, which
  allows you to load images from PNG data. This is a very convenient
  feature, but not necessarily a requirement for using the crate, so
  it's optional. Also, it's a noticeable bump in executable size, so
  use with caution, if you're going for minimal exes.
- The **text** feature implements the `text` mod, which has
  functionality for drawing strings, including multi-line wrapping,
  text alignment, and glyph caching. It does not provide any
  functionality for rasterizing glyphs from fonts.
  - The **font8x8** feature provides text rendering with the
    [`font8x8`](https://crates.io/crates/font8x8) font, via
    `FontHandle::from_font8x8`.
  - The **ttf** feature provides text rendering with any TTF font
    (rasterized by `rusttype`), via `FontHandle::from_ttf`. Crate
    recommendation for getting ttfs from the system:
    [`font-loader`](https://crates.io/crates/font-loader).
- The **profiler** feature implements the `profiler` mod.
  Everything in the `minreq::profiler` module is a no-op if this
  feature is disabled.

## Issues / contributing
If you come across bugs, other issues or feature requests, feel free
to open an issue on
[GitHub](https://github.com/neonmoe/fae/issues/new). Pull requests are
welcome as well, though keep in mind that this is supposed to be a
relatively minimalistic crate, so I probably won't include any
considerable new functionality. If in doubt, open an issue!

## License
The `fae` crate is distributed under the MIT license.
