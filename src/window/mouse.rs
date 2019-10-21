/// Describes a mouse button.
// TODO: Remove this, use glutin mousebutton
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Mouse {
    /// The left mouse button.
    Left,
    /// The right mouse button.
    Right,
    /// The middle mouse button (scroll).
    Middle,
    /// Other mouse buttons, identified by a relatively arbitrary
    /// number.
    Other(u8),
}
