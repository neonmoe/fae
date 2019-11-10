# Fae
Fae is a simple, performant, and compatible 2D rendering crate built
on top of `glutin`, with optional text rendering functionality.

Fae's main design goals are simplicity and performance while
supporting older/low-end target platforms. The optional text-rendering
feature `text` can draw simple text with the `font8x8` crate, or more
sophisticated TTF-based text via `font-kit` (WIP).

Fae supports OpenGL 2.1 and OpenGL ES 2.0 contexts, but will do some
optimizations (VAOs, instanced rendering) if a newer context
(3.3+/ES 3.0+) is available.

This is not a serious contender intended to replace any general 2D
rendering crates. I'm developing this as an exercise to learn about
OpenGL and font rendering, with an eventual goal of being usable for
my own applications. Use with caution!

## Important note
Fae is currently under development, and I wouldn't recommend it for
any kind of usage yet. It's on crates.io mostly so I don't have to
come up with another name :)

## Cargo features
- The **png** feature implements the `Image::from_png` function, which
  allows you to load images from PNG data. This is a very convenient
  feature, but not necessarily a requirement for using the crate, so
  it's optional. Also, it adds some executable size, so use with
  caution, if you're going for minimalism.
- The **text** feature implements the `text` mod, which has
  functionality for drawing strings. If `font8x8` is enabled as a
  feature (which it is, by default), you can draw text with a simple
  8x8 bitmap-based font. If `font-kit` is enabled, you can draw text
  with any font provided by it (WIP).
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
