#!/usr/bin/env bash
# Builds the .rpm inside a Fedora 42 container (GTK 4.18, pango 1.56).
set -euo pipefail
cd "$(dirname "$0")/../.."
dnf install -y -q gcc pkgconf-pkg-config gtk4-devel rpm-build curl python3 git >/dev/null
packaging/prepare-toolchain.sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release -p five-gtk
version="$(packaging/version.sh)"
top="$(mktemp -d)"
mkdir -p "$top"/{SOURCES,SPECS,BUILD,RPMS,SRPMS}
cp packaging/stage.sh packaging/dev.five.files.desktop packaging/dev.five.files.svg "$top/SOURCES/"
cp target/release/five-gtk "$top/SOURCES/"
export FIVE_ROOT="$PWD"
rpmbuild -bb --define "_topdir $top" --define "pkg_version $version" packaging/rpm/five.spec
mkdir -p packaging/out
cp "$top"/RPMS/x86_64/*.rpm packaging/out/
