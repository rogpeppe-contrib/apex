# Copying the same terminal text again doesn't put it back on the clipboard

**Type:** bug
**Area:** terminal, snarf, clipboard

## Summary

A copy from a terminal window only reaches the system clipboard when it
changes Apex's snarf buffer. If the clipboard has been set by another
program since Apex last copied text X, copying X again from a terminal
leaves the other program's text on the clipboard. Pasting in another
application then gives the wrong text, with no sign that the copy
failed. Copies from text windows and tags are not affected: they write
the clipboard every time.

## Steps to reproduce

1. Start Apex and open a terminal with `Newterm`.
2. Run `clear; echo alpha beta gamma`.
3. Double-click `beta` and press Ctrl+Shift+C. The clipboard holds `beta`.
4. In another program, copy something else, for example
   `echo OTHER | xclip -selection clipboard`.
5. Double-click `beta` again and press Ctrl+Shift+C.
6. Paste in another program, or run `xclip -o -selection clipboard`.

The B1-B2 chord (sweep `beta gamma` with B1, click B2) and `Snarf` (typed
in the terminal's tag and clicked with B2 after a sweep) behave the
same in steps 3 and 5.

## Expected behaviour

The clipboard holds `beta` after step 5. Every copy puts its text on the
clipboard, whatever the clipboard held before.

acme does this: `cut` in plan9port src/cmd/acme/exec.c ends with
`putsnarf()` whenever it snarfs, and `putsnarf` (acme.c) writes the whole
snarf buffer to the clipboard (`putsnarf`/`_xputsnarf` in devdraw)
without first checking whether it changed (from memory). So every Snarf
or Cut, and every B1-B2 chord, writes the clipboard.

## Actual behaviour

Measured in nested Xephyr with a debug build of `main` at 3402802,
reading `xclip -o -selection clipboard` after each step:

| Copy | Snarf buffer before | Clipboard before | Clipboard after |
|---|---|---|---|
| terminal, Ctrl+Shift+C on `beta` (first time) | other text | – | `beta` |
| terminal, Ctrl+Shift+C on `beta` again | `beta` | `OTHER` | `OTHER` |
| terminal, B1-B2 chord over `beta gamma` (first time) | other text | – | `beta gamma` |
| terminal, the same chord again | `beta gamma` | `OTHER` | `OTHER` |
| terminal, `Snarf` in the tag, same sweep | `beta gamma` | `OTHER` | `OTHER` |
| terminal, `Snarf` on `alpha` (new text) | `beta gamma` | `OTHER` | `alpha` |
| terminal, `Snarf` on `alpha` again | `alpha` | `OTHER` | `OTHER` |
| text window, Ctrl+C on `cache` again | `cache` | `OTHER` | `cache` |
| text window, B1-B2 chord | – | `OTHER` | the cut text |
| text window, `Snarf` (B2 in the column tag) again | `.cache/` | `OTHER` | `.cache/` |

So only terminal copies are affected, and only when the text is the same
as the snarf buffer already holds.

Apex does not use the X PRIMARY selection at all. After any of these
copies no client owns PRIMARY (`xclip -o -selection primary` finds
nothing), and the client has no `write_to_primary` call. So PRIMARY is
not part of this bug, nor a way around it.

## Where to look

- `crates/apex-client/src/app.rs:App::settle_snarf`. A terminal copy
  can't write the clipboard straight away, because the text comes from the
  server, which holds the scrollback. `App::term_snarf` saves the snarf
  buffer as it was (`self.snarf_wanted = Some(layout.snarf.clone())`) and
  sends `ClientMsg::TermText` (or calls `Server::term_text` for the local
  backend). The server proposes `Proposal::Snarf { text }`, and
  `settle_snarf` (called from `term_snarf` itself and after every
  incoming batch in `main.rs`) writes the clipboard only when
  `layout.snarf != prev`. When the server's text is the same as the
  old snarf buffer, that test never passes, so the clipboard isn't
  written.
- Every terminal copy path goes through `term_snarf`, so all are
  affected: the B1-B2 chord and Alt during a sweep (`chord`, the
  `term_drag` branch, via `term_copy`), `Snarf` in `execute`, the
  Copy/Cut menu and Ctrl+Shift+C (`menu_edit`, `"copy" | "cut"`), and
  ⌘⇧C (`term_copy_last`, the last command's output; from reading the
  code, not tested). `snarf_output` (`Snarfout`) uses the same
  `snarf_wanted` and has the same bug when the output is the same as
  the snarf buffer (from reading the code).
- Not affected: `App::snarf`/`cut` (body and tag: Ctrl+C/X, menus, the
  body B1-B2 chord, typing over a selection), `Snarf`/`Cut` in `execute`
  for non-terminal windows (writes `layout.snarf` after `exec`),
  `snarf_head` (`tagedit.rs`, the path in a tag), `Send`, and the
  overlay field copy (`shell.rs`). All call `cx.write_to_clipboard`
  directly every time.
- A side effect: when the test fails, `snarf_wanted` stays set (it is
  cleared only on a change). It also stays set when the server proposes
  nothing (`Server::term_text` returns `None` for empty text). The
  next unrelated change to the session's snarf buffer is then written
  to this UI's clipboard. That change could be another UI's body snarf
  on the same session, on another machine. Usually harmless, but wrong.

Fix (small): stop using a change in the snarf buffer to tell when the
copy has arrived. The server already has a way to put text on a UI's
clipboard: `ServerMsg::Clipboard`, which lands in `App::clips` and which
`settle_snarf` writes without comparing (it is used for OSC 52 and for
tools' `Proposal::Snarf`, `daemon.rs`). So:

- in `daemon.rs`, `ClientMsg::TermText`: as well as proposing the
  `Snarf`, send `ServerMsg::Clipboard { text }` to the UI that asked
  (`id`);
- for `Backend::Local` in `term_snarf`: take the text out of the
  returned `Proposal::Snarf` and push it onto `self.clips` (or write it
  directly) before `perform`;
- in `snarf_output`: push `out` onto `self.clips` instead of setting
  `snarf_wanted`;
- then remove `snarf_wanted` and its comparison from `settle_snarf`.

Another way, keeping the client side only: make `snarf_wanted` a count
of `LayoutOp::Snarf` entries seen, not the old text, and write the
clipboard when the next one arrives. But the reply message is simpler,
and it can't pick up another UI's snarf by mistake.

## Notes

- Scope: upstream shared code. `snarf_wanted` came in with 3a0e86c
  ("copies come from the server") and was extended by 6a6e9e1
  (Snarfout), both before the Linux commits 811a3d0 and 96ca2bf. There
  is no `cfg(target_os)` on the path, apart from Ctrl+Shift+C itself,
  which is Linux-only. macOS behaves the same with ⌘C, the menu, the
  chord and `Snarf` (high confidence, not tested there). On macOS the
  other program would be anything that sets the pasteboard.
- First noticed while testing issue 27 (FIXED/27-term-line-copy-drops-newline.md,
  commit 928efac), where a second copy of `last` left `none` on the
  clipboard.
- Related: issue 21 (terminal chords), issue 26 (typing over a selection
  snarfs it, which goes through `App::snarf` and is not affected).
