#!/usr/bin/env bash
# Installs FIVE from the extracted tarball. Default: ~/.local ; use PREFIX=/usr/local with sudo for a system-wide install.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
prefix="${PREFIX:-$HOME/.local}"
cp -r "$here/root/usr/." "$prefix/"
command -v update-desktop-database >/dev/null && update-desktop-database "$prefix/share/applications" || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q "$prefix/share/icons/hicolor" || true
echo "FIVE installed in $prefix (run: five-gtk). Requires GTK 4.16 or newer."
