#[cfg(not(feature = "font8x8"))]
mod dummy;
#[cfg(feature = "font8x8")]
mod font8x8;

#[cfg(feature = "font8x8")]
pub use self::font8x8::Font8x8Provider;
#[cfg(not(feature = "font8x8"))]
pub use dummy::DummyProvider;
