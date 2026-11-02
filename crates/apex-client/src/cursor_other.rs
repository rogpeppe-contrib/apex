//! The pointers on a platform other than the Mac (`cursor.rs` is the
//! Mac's): the system's own throughout. acme's big arrow and box are
//! not drawn here yet, and a page keeps whatever gpui's hovered style
//! says, there being no cursor rect of the window's to silence.

/// The style over a web or preview body: the arrow, which the page's
/// view replaces with its own while the pointer is in it.
pub const NATIVE_CURSOR: gpui::CursorStyle = gpui::CursorStyle::Arrow;

/// Set the system cursor for a style now, over a page: nothing, the
/// page's view having the pointer while it is over it.
pub fn apply(_style: gpui::CursorStyle) {}

/// Nothing to put behind the styles.
pub fn install() {}
