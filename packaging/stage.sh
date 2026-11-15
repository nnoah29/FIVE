#!/usr/bin/env bash
# Usage: stage.sh DEST PREFIX — installs the release binary, desktop entry and icon under DEST/PREFIX.
set -euo pipefail
root="${FIVE_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
dest="$1"; prefix="${2:-/usr}"
install -Dm755 "$root/target/release/five-gtk" "$dest$prefix/bin/five-gtk"
install -Dm644 "$root/packaging/dev.five.files.desktop" "$dest$prefix/share/applications/dev.five.files.desktop"
install -Dm644 "$root/packaging/dev.five.files.svg" "$dest$prefix/share/icons/hicolor/scalable/apps/dev.five.files.svg"
