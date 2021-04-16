# fae
Fae is a hardware-accelerated 2D sprite renderer, with optional text
rendering capabilities.

The rendering is implemented with OpenGL 2.1/3.3 depending on your
hardware. Font rasterization uses `rusttype` for ttfs, and `font8x8`
can be used as a small fallback font.

Why you **shouldn't** use this crate: I haven't profiled this crate
against other similar crates, nor do I have the experience to know if
this crate is a good implementation of a 2D renderer. I'm making it to
power my own games, because of NIH.

Update 2021-04-16: I personally use SDL's render module for the
usecase I originally built this for. So, consider
[rust-sdl2](https://crates.io/crates/sdl2)!

## [Documentation][docs]
See the [`examples/`](examples/) as well.

## Optional features
- The `text` feature provides access to the text rendering API, but
  requires `font8x8` or `ttf` to be enabled as well.
  - The `font8x8` feature provides access to text rendering with the
    [font8x8][font8x8] font.
  - The `ttf` feature provides access to text rendering with ttf fonts
    via the [rusttype][rusttype] crate.
- The `png` feature provides easy png loading functionality via the
  [png][png] crate.

## License
This library is provided under the terms of the [MIT
license][license].

[docs]: https://docs.rs/fae
[font8x8]: https://crates.io/crates/font8x8
[rusttype]: https://crates.io/crates/rusttype
[png]: https://crates.io/crates/png
[license]: LICENSE.md
