#!/usr/bin/env bash
# Builds the .deb and the generic tarball inside a Debian 13 (trixie) container; glibc 2.41 and GTK 4.18 are the baseline.
set -euo pipefail
cd "$(dirname "$0")/../.."
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq build-essential curl ca-certificates pkg-config libgtk-4-dev dpkg-dev python3 git >/dev/null
packaging/prepare-toolchain.sh
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
cargo build --release -p five-gtk
version="$(packaging/version.sh)"
out="packaging/out"; mkdir -p "$out"
work="$(mktemp -d)"
packaging/stage.sh "$work/pkg" /usr
mkdir -p "$work/pkg/DEBIAN"
size="$(du -sk "$work/pkg/usr" | cut -f1)"
cat > "$work/pkg/DEBIAN/control" <<CONTROL
Package: five
Version: $version
Section: utils
Priority: optional
Architecture: amd64
Depends: libgtk-4-1 (>= 4.16), libglib2.0-0, libpango-1.0-0 (>= 1.56)
Installed-Size: $size
Maintainer: nnoah29 <noreply@users.noreply.github.com>
Homepage: https://github.com/nnoah29/FIVE
Description: FIVE, a fast native file manager
 Folder cards, list and grid views, previews, bulk rename, zip, duplicates
 and an automation rules engine. Built with Rust and GTK 4.
CONTROL
dpkg-deb --build --root-owner-group "$work/pkg" "$out/five_${version}_amd64.deb"
tar_dir="five-$version-linux-x86_64"
mkdir -p "$work/$tar_dir"
packaging/stage.sh "$work/$tar_dir/root" /usr
cp packaging/install.sh README.md "$work/$tar_dir/"
tar -C "$work" -czf "$out/$tar_dir.tar.gz" "$tar_dir"
