# A B1 sweep dragged out of the app window stops following the pointer: no autoscroll below it, and the scroll rate freezes

**Type:** bug
**Area:** mouse (B1 selection sweep / autoscroll)

## Summary

Since FIXED/16, sweeping a B1 selection and dragging below a body autoscrolls,
as acme does. That only works while the pointer stays inside the Apex OS
window, though. Once the pointer leaves the window, Apex sees no more motion
until the button is let go. A body that ends at the bottom of the window (any
bottom-most window in a column) therefore never autoscrolls when you drag out
below the window. If autoscroll had already started inside the window, it keeps
going, but at the rate set by the last position seen inside, however far out
the pointer goes. The same holds at the top edge and the sides. Dragging out
past the right edge and then up or down leaves the selection where it was.
Releasing outside the window does end the sweep properly.

## Steps to reproduce

Xephyr at 1200x800. The apex window is moved to (100,100) and sized to
1000x500, so there is screen space all round it:
`xdotool windowmove $W 100 100 windowsize $W 1000 500`.

1. `seq 1 500 > long.txt; apex open long.txt`. It opens as the lower window
   of the right-hand column. Its body ends at about y=597, and the window's
   bottom edge is at y=600.
2. **Out below, no autoscroll:** press B1 on line 5 (760,340) and drag down in
   40 px steps, ...590, 620, 660, 700. The pointer is now 100 px below the
   window. Hold for 2 s, then release out there.
3. **Started inside, then out:** click to reset. Press on line 5 and drag down
   to y=599. That is inside the window, in the 2–3 px strip under the body.
   Hold 1 s, then jump to y=700 and hold 1 s more.
4. **Top edge:** with the window scrolled, press at y=400, drag up to y=240
   (just above the body top, about 23 px), hold 0.5 s, then jump to y=40,
   above the window, and hold 0.5 s.
5. **Side:** press at (760,340), drag right out of the window to x=1150 at the
   same height, then down to y=580 while still outside.

## Expected behaviour

acme keeps tracking the pointer wherever it goes while a button is down. X
gives the client an implicit pointer grab from the press until the release, and
libframe's `frselect` (plan9port src/libframe/frselect.c, from memory) loops
on `readmouse` and calls the scroll function with
`(pt.y - f->r.max.y)/font->height + 1` whenever the pointer is below the frame,
or above it, wherever on the screen that is. So:

- Dragging below the window scrolls, faster the further out the pointer goes,
  until the end of the text is in view (FIXED/16).
- At the top it scrolls up at a rate that follows the pointer.
- Off to the side, vertical motion still moves the sweep's end line by line.
  The column is clamped to the line's start or end.

## Actual behaviour

`apex sel` on long.txt, as `q0 q1`:

- **Step 2 (out below):** `9 50` right after leaving, and still `9 50` after
  2 s. Nothing scrolls. The sweep ends at about line 19, where the last motion
  inside the window (y=590) left it. After the release outside it is still
  `9 50`, and moving back into the window afterwards changes nothing. So the
  release *is* seen, and the sweep ends cleanly.
- **Step 3 (started inside):** at y=599, `9 86` after 1 s (one line per
  80 ms tick, since the pointer is about 2 px past the body). After jumping
  to y=700: `9 125`, then `9 158` a second later. That is still about one line
  per tick, though the pointer is now about 100 px (6 lines) past the body.
  On release it stops (`9 164`, then steady).
- **Step 4 (top):** `134 134`, then `92 134` after 0.5 s just above the body.
  After jumping to y=40, `74 134` after 0.5 s more. The rate stayed the same,
  though the pointer was about 220 px above the body, which acme would turn
  into about 13 lines per step. The release above the window ends the sweep.
- **Step 5 (side):** `122 134` once outside to the right, and still
  `122 134` after moving down 180 px outside. The sweep doesn't follow at all.

In a maximised window the pointer can't go past the screen's bottom row. That
row is inside the window, in the strip under the body, so there you get the
slowest scroll, one line per tick, and can't speed it up.

## Where to look

- `crates/apex-client/src/main.rs`, the root view's `render` (around line
  233): `.on_mouse_move(cx.listener(Self::mouse_move))`. In gpui,
  `Interactivity::on_mouse_move` (gpui `src/elements/div.rs`, around line 303)
  calls the listener only `if phase == DispatchPhase::Bubble &&
  hitbox.is_hovered(window)`. `HitboxId::is_hovered` (gpui `src/window.rs`,
  around line 774) is a hit test of `window.mouse_position` against the last
  frame's hitboxes, done in `Window::dispatch_mouse_event` (around
  line 5548). A position outside the window hits no hitbox, so
  `Acme::mouse_move` (`crates/apex-client/src/app.rs`, around line 3789) is
  never called, and `last_mouse` stays at the last position inside.
- The events do arrive in gpui; they are dropped only at that hit test. On
  X11, gpui_linux `x11/window.rs` (around line 706) selects XI2
  `MOTION | BUTTON_PRESS | BUTTON_RELEASE | ENTER | LEAVE` on the window.
  During the implicit grab from a button press, X sends motion to that window
  with coordinates outside its bounds (negative, or larger than the window).
  `x11/client.rs`, `Event::XinputMotion` (around line 1248), turns every one
  into `PlatformInput::MouseMove` with no bounds check, and `Window::dispatch_event`
  sets `mouse_position` from it (`window.rs`, around line 5237). The `XinputLeave`
  handler only sends `MouseExited` and calls `set_hovered(false)`. (Checked in the
  gpui checkout in `~/.cargo/git/checkouts/zed-*/5a9b955`.)
- Autoscroll (`app.rs`: the B1 branch of `mouse_move`, around line 3911, and
  `autoscroll_step`, around line 4440) works out `above`/`below` and the
  distance from `self.last_mouse`. So without motion events it never starts
  when the edge is crossed outside the window, and the timer loop, once going,
  keeps using the stale distance.
- Release works because the root also has `.on_mouse_up_out(MouseButton::Left,
  …)` (main.rs, around line 220). That is a capture-phase listener for "not
  hovered", so `mouse_up` runs and clears `mouse.b1` and `mouse.autoscroll`.
  There is no `on_mouse_move` equivalent of `_out` on `Div`.
- Possible fixes:
  - gpui has `Window::capture_pointer(hitbox_id)` (window.rs, around
    line 2971). While a hitbox is captured, it counts as hovered for every
    event, and the capture is released on mouse up. Calling it on a B1 (or
    B2/B3) press with the root div's hitbox would route the outside motion to
    `mouse_move`. The catch is that the root `Div`'s hitbox id isn't easy to
    get from the view.
  - Simpler: register a raw listener with `window.on_mouse_event::<MouseMoveEvent>`
    during paint (for example from a `canvas` child or one of Apex's own
    elements). It sees every event with no hit test, and can forward to
    `mouse_move` when the event lands outside the window's bounds while a
    sweep is live.

  Either way it's a small change.

## Notes

- Reproduced on Linux under X11 (Xephyr 1200x800, no window manager, apex
  window 1000x500 at 100,100) with the prebuilt
  `target/apex-linux/bin/apex-ui` from `main` (05b3706).
- This is not Linux-only code. The handler wiring in main.rs and gpui's hit
  test are shared, and nothing in commits 811a3d0/96ca2bf touches them. On
  macOS, AppKit sends `mouseDragged:` to the window outside its frame as well,
  and gpui_macos makes those into `MouseMove` events too. So the same drop
  almost certainly happens there (not checked on a Mac).
- The same loss of tracking applies to anything else that follows the pointer
  through `mouse_move` while a button is down: B2/B3 sweeps (`hl`), terminal
  selection drags, and sweeps in a tag's edit field. They all freeze at the
  last position inside the window. Window and column drags (`box_drag`) are
  unaffected, since they act on the release position, which `on_mouse_up_out`
  delivers.
- When the pointer comes back into the window with the button still down,
  tracking resumes at once.
- Related: FIXED/16-selection-autoscroll-overrun.md. Its "Notes" and "Fix"
  sections already point out this case as not fixed.

## Fix

`crates/apex-client/src/main.rs` (the root view's `render`) adds a zero-size
`canvas` whose paint registers a raw `window.on_mouse_event::<MouseMoveEvent>`
listener. In the bubble phase, for a move with a button held
(`pressed_button.is_some()`) whose position lies outside the window's viewport,
it calls `Acme::mouse_move`. Moves inside the window still come only through
the root's `on_mouse_move`, so none is delivered twice. Body sweeps now
autoscroll below and above the window, faster the further out the pointer is,
and follow vertical motion off either side. B2/B3 sweeps, terminal drags and
scrollbar drags follow the pointer there too. Releasing outside still ends the
drag. Verified in Xephyr with the steps above. The fix is in the commit that
contains this section.
