#[cfg(feature = "clipboard")]
mod actual;
#[cfg(feature = "clipboard")]
pub use self::actual::*;
#[cfg(not(feature = "clipboard"))]
mod placeholder;
#[cfg(not(feature = "clipboard"))]
pub use self::placeholder::*;
