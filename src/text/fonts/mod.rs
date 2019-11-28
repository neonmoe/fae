#[cfg(feature = "font8x8")]
pub mod font8x8;
#[cfg(feature = "font8x8")]
pub(crate) use self::font8x8::Font8x8Provider;

#[cfg(feature = "ttf")]
mod rusttype;
#[cfg(feature = "ttf")]
pub(crate) use self::rusttype::RustTypeProvider;
