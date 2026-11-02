//! Moving the mouse, as acme does after layout changes (`moveto`). macOS
//! delivers no event for a warp, so the app remembers where it put the
//! pointer until the next real mouse event. (X delivers one, at the same
//! place; Wayland allows no warp at all.)

use gpui::{Pixels, Point, Window};

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWarpMouseCursorPosition(p: CGPoint) -> i32;
    fn CGAssociateMouseAndMouseCursorPosition(connected: bool) -> i32;
}

/// Put the pointer at `p`, a point in this window's coordinates.
#[cfg(target_os = "macos")]
pub fn move_to(window: &Window, p: Point<Pixels>) {
    let p = screen_point(window, p);
    // SAFETY: plain CoreGraphics calls with a by-value point.
    unsafe {
        CGWarpMouseCursorPosition(p);
        // a warp otherwise suppresses mouse movement for a moment
        CGAssociateMouseAndMouseCursorPosition(true);
    }
    // AppKit hides the pointer while keys are typed, until the mouse
    // moves; a warp is not a move, so a pointer put somewhere from the
    // keyboard (ctrl-tab, a Goto) would sit there unseen
    show();
}

#[cfg(target_os = "macos")]
fn screen_point(window: &Window, p: Point<Pixels>) -> CGPoint {
    use objc::{class, msg_send, sel, sel_impl};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let fallback = || {
        let b = window.bounds();
        CGPoint {
            x: f32::from(b.origin.x + p.x) as f64,
            y: f32::from(b.origin.y + p.y) as f64,
        }
    };

    let Ok(h) = HasWindowHandle::window_handle(window) else {
        return fallback();
    };
    let RawWindowHandle::AppKit(h) = h.as_raw() else {
        return fallback();
    };
    let view = h.ns_view.as_ptr() as *mut objc::runtime::Object;

    // SAFETY: gpui gives us its live NSView. These are AppKit geometry queries
    // on the main thread. AppKit screen coordinates are y-up; CoreGraphics
    // cursor warps use y-down global display coordinates.
    unsafe {
        let ns_window: *mut objc::runtime::Object = msg_send![view, window];
        if ns_window.is_null() {
            return fallback();
        }
        let screens: *mut objc::runtime::Object = msg_send![class!(NSScreen), screens];
        let main_screen: *mut objc::runtime::Object = msg_send![screens, objectAtIndex: 0usize];
        if main_screen.is_null() {
            return fallback();
        }
        let view_frame: CGRect = msg_send![view, frame];
        let in_window = CGPoint {
            x: view_frame.origin.x + f32::from(p.x) as f64,
            y: view_frame.origin.y + view_frame.size.height - f32::from(p.y) as f64,
        };
        let screen: CGPoint = msg_send![ns_window, convertPointToScreen: in_window];
        let main_frame: CGRect = msg_send![main_screen, frame];
        CGPoint {
            x: screen.x,
            y: main_frame.origin.y + main_frame.size.height - screen.y,
        }
    }
}

/// Put the pointer at `p`, a point in this window's coordinates: under
/// X11 a `WarpPointer` on gpui's own connection, in the window's device
/// pixels. X reports a warp as a motion like any other, at the point it
/// was put, which the app's memory of where it put it agrees with.
/// Wayland lets no client move the pointer: there it stays where it was.
#[cfg(target_os = "linux")]
pub fn move_to(window: &Window, p: Point<Pixels>) {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::ConnectionExt;

    let Ok(w) = HasWindowHandle::window_handle(window) else { return };
    let RawWindowHandle::Xcb(w) = w.as_raw() else { return };
    let Ok(d) = HasDisplayHandle::display_handle(window) else { return };
    let RawDisplayHandle::Xcb(d) = d.as_raw() else { return };
    let Some(c) = d.connection else { return };
    // SAFETY: the connection is gpui's, alive as long as the window that
    // handed it over, which outlives this call; the wrapper is told not
    // to disconnect it when dropped. libxcb is safe to share.
    let Ok(conn) = (unsafe { x11rb::xcb_ffi::XCBConnection::from_raw_xcb_connection(c.as_ptr(), false) }) else { return };
    // gpui's pixels are the window's device pixels over its scale
    let s = window.scale_factor();
    let (x, y) = ((f32::from(p.x) * s).round() as i16, (f32::from(p.y) * s).round() as i16);
    let _ = conn.warp_pointer(x11rb::NONE, w.window.get(), 0, 0, 0, 0, x, y);
    let _ = conn.flush();
}

/// Elsewhere the pointer stays where it was.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn move_to(_window: &Window, _p: Point<Pixels>) {}

/// The pointer shown, where it is: AppKit's typing-hides-it undone.
#[cfg(target_os = "macos")]
pub fn show() {
    use objc::{class, msg_send, sel, sel_impl};
    // SAFETY: a class method on NSCursor, on the main thread.
    unsafe {
        let _: () = msg_send![class!(NSCursor), setHiddenUntilMouseMoves: false];
    }
}
