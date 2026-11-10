# Apex issues

Reported on Linux/X11 against the `001-linux-support` build. Each file has
steps to reproduce, expected vs actual behaviour, screenshots in `images/`,
and a pointer to the likely code.

| # | Issue | Type |
|---|-------|------|
| 01 | [Terminal: B1 sweep ignored when it starts soon after a click](01-term-click-drag.md) | bug |
| 02 | [Renaming a window in its tag is discarded unless you press Return](02-tag-title-edit-not-persisted.md) | bug |
| 03 | [B1 dragged below a tag stops at the pointer's column, not the tag's end](03-tag-drag-down-selection.md) | bug |
| 04 | [Support acme's Alt compose sequences for typing Unicode](04-unicode-compose-input.md) | feature |
| 05 | [Edit's pipe and file commands are not implemented (`Edit ,>diff $% -`)](05-edit-pipe-diff-filename.md) | feature (missing acme behaviour) |
| 06 | [Paste chord (B1-B3) on the window's name in the tag does nothing](06-paste-chord-tag-filename.md) | bug |
| 07 | [B2 on the window's path in the tag does nothing](07-execute-text-in-tag-filename.md) | bug |
| 08 | [A web window's header has no Del; Ctrl+W over the page does nothing](08-cannot-delete-web-window.md) | bug |
| 09 | [Button 2 in a Term window's scrollbar does nothing](../FIXED/09-term-scrollbar-middle-button.md) | **fixed**: B2 in a Term scrollbar jumps and drags, as in text windows |
| 10 | [B3 on a soft-wrapped file name in a terminal doesn't open it](10-term-plumb-wrapped-filename.md) | bug |
| 11 | [B1-B2 chord with null selection doesn't make the window puttable](11-chord-null-selection-put.md) | bug |
| 12 | [Double-click just before a closing bracket in a terminal doesn't match](12-double-click-before-close-paren.md) | bug |
| 13 | [The file name in the tag can't be selected or copied](13-tag-filename-select-copy.md) | bug |
| 14 | [Starting a daemon removes a live daemon's socket](14-daemon-removes-live-socket.md) | bug |
| 15 | [Escape does not cut a non-empty selection](15-escape-on-selection.md) | bug |
| 16 | [Selection autoscroll runs past the end of text and speeds up](16-selection-autoscroll-overrun.md) | bug |
| 17 | [Double-click then drag should extend by words](17-double-click-drag-by-words.md) | feature |
| 18 | [Triple-click should select the line; drag extends by lines](18-triple-click-drag-by-lines.md) | feature |
| 19 | [Language-aware double-click selection](19-language-aware-selection.md) | feature |
| 20 | [Editing the window's name in the tag behaves unlike editing text](20-tag-filename-editing-weird.md) | bug / design |
| 21 | [B1-B2-B3 chord in a terminal types the selection at the prompt](../FIXED/21-term-chord-cut-paste-copies.md) | **fixed**: B3 after B2 in the same terminal sweep no longer types into the shell |
| 22 | [Tab-separated columns don't line up in the proportional font](22-tabs-misaligned.md) | bug |

Related groups: 02, 06, 07, 13, 20 (the tag's name area); 01, 10, 12, 21 (terminal
mouse handling); 03, 16 (B1 sweeps past the edge of the text); 12, 17, 18, 19 (double-click expansion).
