#!/usr/bin/env bash
# Installs FIVE from the extracted tarball. Default: ~/.local ; use PREFIX=/usr/local with sudo for a system-wide install.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
prefix="${PREFIX:-$HOME/.local}"
cp -r "$here/root/usr/." "$prefix/"
service="$prefix/share/dbus-1/services/org.freedesktop.impl.portal.desktop.five.service"
[ -f "$service" ] && sed -i "s|^Exec=.*|Exec=$prefix/bin/five-gtk --portal|" "$service"
command -v update-desktop-database >/dev/null && update-desktop-database "$prefix/share/applications" || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q "$prefix/share/icons/hicolor" || true
echo "FIVE installed in $prefix (run: five-gtk). Requires GTK 4.16 or newer."
echo "To use FIVE as your file chooser: five-gtk --enable-chooser (or Settings, System)."
