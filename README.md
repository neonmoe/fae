# Fae
Fae is a simple, performant and compatible 2D rendering crate with
optional window creation functionality and text rendering. Its main
design goals are simplicity and performance while supporting
older/low-end target platforms. The base crate which implements the
rendering functions only depends on OpenGL and std. Optional features
exist for ttf rendering and window creation, `rusttype` and
`glutin`/`glfw` respectively. The crate supports OpenGL 2.1+ and
OpenGL ES 2.0+ contexts, but will do some optimizations if a 3.3 or ES
3.0 context is available.

## Important note
The crate is currently under development, and I wouldn't recommend it
for any kind of usage yet. Especially since the API is currently
oriented so it fits the backend of the crate, instead of being easy to
use, and that will definitely change in the future. It's on crates.io
mostly so I don't have to come up with another name :)

## Cargo features
- The **glutin** feature implements the `window` mod, which allows for
  easy window creation using glutin, with all the required OpenGL
  context wrangling done for you.
- The **glfw** feature is an alternative to the **glutin** feature,
  implementing the same API. Linux executables based on **glfw** are
  considerably smaller than **glutin** ones (1.1MB vs 2.5MB, stripped
  and built with `--release`), which is why I wrote this alternative.
  GLFW builds do seem to be slightly heavier on the performance side
  though. Do your own benchmarks and use the one that fits your
  project. **Important notes**:
  - Glutin and GLFW handle DPI differently in x11 contexts.
  - The `is_dialog` variable of `WindowSettings` only works in x11
    contexts with Glutin.
- The **text** feature implements the `text` mod, which has
  functionality for drawing strings. Fonts are provided in the form of
  .ttf files shipped with your application. The glyph rendering is
  done by `rusttype`, which this feature adds as a dependency, as well
  as `unicode-normalization`. A lightweight version of this feature is
  planned, where you can use bitmap fonts to conserve executable size
  and performance.
- The **png** feature implements the `Image::from_png` function, which
  allows you to load images from PNG data. This is a very convenient
  feature, but not necessarily a requirement for using the crate, so
  it's optional.

## Notes
- You can force the crate to use an OpenGL 2.1 context by setting the
  `FAE_OPENGL_LEGACY` environment variable. This is intended for
  making sure that builds work in both legacy and modern modes, though
  of course, if there are visual differences between the modes (aside
  from fps), they should be considered a bug in this crate. Make an
  issue! Example usage:
  ```sh
  FAE_OPENGL_LEGACY=1 cargo run
  ```

## License
The `fae` crate is distributed under the MIT license.
