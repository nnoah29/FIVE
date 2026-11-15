#!/usr/bin/env bash
# Usage: stage.sh DEST PREFIX — installs the release binary, desktop entry and icon under DEST/PREFIX.
set -euo pipefail
root="${FIVE_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
dest="$1"; prefix="${2:-/usr}"
install -Dm755 "$root/target/release/five-gtk" "$dest$prefix/bin/five-gtk"
install -Dm644 "$root/LICENSE" "$dest$prefix/share/licenses/five/LICENSE"
install -Dm644 "$root/packaging/dev.five.files.desktop" "$dest$prefix/share/applications/dev.five.files.desktop"
install -Dm644 "$root/packaging/portal/five.portal" "$dest$prefix/share/xdg-desktop-portal/portals/five.portal"
install -Dm644 "$root/packaging/portal/org.freedesktop.impl.portal.desktop.five.service" "$dest$prefix/share/dbus-1/services/org.freedesktop.impl.portal.desktop.five.service"
install -Dm644 "$root/packaging/dev.five.files.svg" "$dest$prefix/share/icons/hicolor/scalable/apps/dev.five.files.svg"
