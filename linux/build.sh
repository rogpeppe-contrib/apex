#!/bin/sh
# Build Apex for Linux into target/apex-linux, and with --install put it
# under ~/.local: the binaries in bin, the desktop entry and the icon
# where the desktop looks for them. Usage: linux/build.sh [--debug] [--install]
#
# Needs: Rust, Zig 0.16 (for libghostty-vt: on PATH, $ZIG, or
# ~/.local/zig-*-0.16*/zig), and the GTK and WebKitGTK development
# packages (on Debian and Ubuntu: libwebkit2gtk-4.1-dev libgtk-3-dev).
set -e
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
profile=release; flag=--release; install=
for a in "$@"; do
  case "$a" in
    --debug) profile=debug; flag= ;;
    --install) install=yes ;;
    *) echo "usage: linux/build.sh [--debug] [--install]" >&2; exit 2 ;;
  esac
done
cargo build $flag -p apex-client -p apex-cli
# rc, the shell commands run with (mariusae/rustrc); cargo install builds out of tree
cargo install --git https://github.com/mariusae/rustrc --bin rc --root target/rc-host --force --quiet

# the three beside one another: apex-ui finds `apex` and `rc` beside
# itself, and so do remote hosts of this machine's own kind
out=target/apex-linux
rm -rf "$out"
mkdir -p "$out/bin" "$out/share/applications" "$out/share/icons/hicolor/scalable/apps"
cp "target/$profile/apex-ui" "$out/bin/apex-ui"
cp "target/$profile/apex" "$out/bin/apex"
ln -sfn apex "$out/bin/apex-editor"
cp target/rc-host/bin/rc "$out/bin/rc"
cp linux/apex.desktop "$out/share/applications/apex.desktop"
cp mac/space-bunny.svg "$out/share/icons/hicolor/scalable/apps/apex.svg"
echo "built $out"

if [ -n "$install" ]; then
  cp -r "$out"/. "$HOME/.local/"
  update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
  gtk-update-icon-cache -q "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
  echo "installed under $HOME/.local (is $HOME/.local/bin on your PATH?)"
fi
