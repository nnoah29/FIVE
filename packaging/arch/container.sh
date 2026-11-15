#!/usr/bin/env bash
# Builds the Arch package inside an archlinux container with makepkg (needs the vX.Y.Z tag to be published).
set -euo pipefail
cd "$(dirname "$0")/../.."
pacman -Syu --noconfirm --needed base-devel git python gtk4 pango rust >/dev/null
useradd -m builder 2>/dev/null || true
work="$(mktemp -d)"
cp packaging/arch/PKGBUILD "$work/"
version="$(packaging/version.sh)"
sed -i "s/^pkgver=.*/pkgver=$version/" "$work/PKGBUILD"
chown -R builder "$work"
su builder -c "cd '$work' && makepkg -f --noconfirm"
mkdir -p packaging/out
cp "$work"/*.pkg.tar.zst packaging/out/
