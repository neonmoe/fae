# fae
*Sprites are a supernatural entities, often depicted as fae-like
creatures.*

Fae is a simple, performant and compatible 2D rendering package with
optional window creation functionality and text rendering. Its main
design goals are simplicity and performance while keeping support for
older target platforms. Rendering is implemented in OpenGL, glyph
drawing is done by `rusttype`, and the window initialization by
`glutin`. The crate supports OpenGL 2.1+ and OpenGL ES 2.0+ contexts,
but will do optimizations if a 3.3 or ES 3.0 context is available.

## Features
As is, without the default features, the crate has the functionality
for drawing rectangles in various ways: rotated, tinted, and
sprited. Ninepatch sprites, ie. sprites with borders defined
separately from the middle part, are supported as well.

The optional features (all enabled by default) consist of the
following:
- The **glutin** feature implements the `window` mod, which allows for
  easy window creation using glutin, with all the required OpenGL
  context wrangling done for you.
- The **text** feature implements the `text` mod, which adds rendering
  strings to the available render functions. Fonts are provided in the
  form of .ttf files shipped with your application. The glyph
  rendering is done by `rusttype`, which this feature adds as a
  dependency, as well as `unicode-normalization`.
- The **png** feature implements the `Image::from_png` function, which
  allows you to load images from PNG data. This is a very convenient
  feature, but not necessarily a requirement for using the crate, so
  it's optional.

## License
The `fae` crate is distributed under the MIT license.
