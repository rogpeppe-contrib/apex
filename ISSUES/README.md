# Apex issues

Reported on Linux/X11 against the `001-linux-support` build. Each file has
steps to reproduce, expected vs actual behaviour, screenshots in `images/`,
and a pointer to the likely code.

| # | Issue | Type |
|---|-------|------|
| 01 | [Terminal: B1 sweep ignored when it starts soon after a click](../FIXED/01-term-click-drag.md) | **fixed**: a drag after a double-click in a terminal sweeps from the press point, as in text windows |
| 02 | [Renaming a window in its tag is discarded unless you press Return](../FIXED/02-tag-title-edit-not-persisted.md) | **fixed**: a tag rename or label edit is committed when you click elsewhere or run a command; only Esc cancels |
| 03 | [B1 dragged below a tag stops at the pointer's column, not the tag's end](../FIXED/03-tag-drag-down-selection.md) | **fixed**: a sweep dragged below a tag runs to the end of its text (above it, to its start), as in acme |
| 04 | [Support acme's Alt compose sequences for typing Unicode](04-unicode-compose-input.md) | feature |
| 05 | [Edit's pipe and file commands are not implemented (`Edit ,>diff $% -`)](05-edit-pipe-diff-filename.md) | feature (missing acme behaviour) |
| 06 | [Paste chord (B1-B3) on the window's name in the tag does nothing](../FIXED/06-paste-chord-tag-filename.md) | **fixed**: the 1-3 chord on the path renames the window to the snarf; chords cut and paste in the rename field |
| 07 | [B2 on the window's path in the tag does nothing](../FIXED/07-execute-text-in-tag-filename.md) | **fixed**: B2 on the tag's path (name or any folder) executes the window's whole name, as acme's B2 on the name does; B2-B1 adds the selection |
| 08 | [A web window's header has no Del; Ctrl+W over the page does nothing](../FIXED/08-cannot-delete-web-window.md) | **fixed**: the web header has the tag's × (Del), and Ctrl+W closes the web window over its header or its page |
| 09 | [Button 2 in a Term window's scrollbar does nothing](../FIXED/09-term-scrollbar-middle-button.md) | **fixed**: B2 in a Term scrollbar jumps and drags, as in text windows |
| 10 | [B3 on a soft-wrapped file name in a terminal doesn't open it](../FIXED/10-term-plumb-wrapped-filename.md) | **fixed**: B2/B3 words, double-click and sweeps in a terminal run across a soft wrap, so a wrapped file name opens |
| 11 | [B1-B2 chord with null selection doesn't make the window puttable](../FIXED/11-chord-null-selection-put.md) | **fixed**: a B1 chord (or Cut/Paste) in a clean file's body marks it changed, so Put appears even when nothing was cut |
| 12 | [Double-click just before a closing bracket in a terminal doesn't match](../FIXED/12-double-click-before-close-paren.md) | **fixed**: a terminal double-click expands from the character boundary nearest the pointer, as in text windows |
| 13 | [The file name in the tag can't be selected or copied](../FIXED/13-tag-filename-select-copy.md) | **fixed**: Copy/Cut/Paste work in the rename field; the 1-2 chord on the path snarfs it |
| 14 | [Starting a daemon removes a live daemon's socket](../FIXED/14-daemon-removes-live-socket.md) | **fixed**: a daemon refuses to start on a path another daemon serves, and removes its socket on exit only if it is still its own |
| 15 | [Escape does not cut a non-empty selection](../FIXED/15-escape-on-selection.md) | **fixed**: Esc cuts a non-empty selection (with snarf), so a second Esc after typing cuts the typed text |
| 16 | [Selection autoscroll runs past the end of text and speeds up](../FIXED/16-selection-autoscroll-overrun.md) | **fixed**: autoscroll below a body stops once the end of the text is in view, and recrossing the edge no longer starts extra scroll loops |
| 17 | [Double-click then drag should extend by words](17-double-click-drag-by-words.md) | feature |
| 18 | [Triple-click should select the line; drag extends by lines](18-triple-click-drag-by-lines.md) | feature |
| 19 | [Language-aware double-click selection](19-language-aware-selection.md) | feature |
| 20 | [Editing the window's name in the tag behaves unlike editing text](../FIXED/20-tag-filename-editing-weird.md) | **fixed**: the rename field takes B1 clicks, sweeps and double-clicks, shows the path as the tag does, stays within the tag row and scrolls; any rename marks a file's window changed (breadcrumb picker kept by design) |
| 21 | [B1-B2-B3 chord in a terminal types the selection at the prompt](../FIXED/21-term-chord-cut-paste-copies.md) | **fixed**: B3 after B2 in the same terminal sweep no longer types into the shell |
| 22 | [Tab-separated columns don't line up in the proportional font](../FIXED/22-tabs-misaligned.md) | **fixed**: tabs reach pixel tab stops (`tabstop` × width of '0') in any font, and `Tab n` redraws the window |

Related groups: 02, 06, 07, 13, 20 (the tag's name area); 01, 10, 12, 21 (terminal
mouse handling); 03, 16 (B1 sweeps past the edge of the text); 12, 17, 18, 19 (double-click expansion).
