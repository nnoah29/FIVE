#!/usr/bin/env bash
# Usage: build-in-container.sh deb|rpm|arch — builds the committed tree (HEAD) in the same image as CI. Output: packaging/out/
set -euo pipefail
cd "$(dirname "$0")/.."
case "${1:-}" in
  deb) image=debian:trixie ;;
  rpm) image=fedora:42 ;;
  arch) image=archlinux:latest ;;
  *) echo "usage: $0 deb|rpm|arch" >&2; exit 2 ;;
esac
mkdir -p packaging/out
git archive HEAD | docker run --rm -i -v "$PWD/packaging/out":/out -e CARGO_HOME=/tmp/cargo -e RUSTUP_HOME=/tmp/rustup "$image" \
  bash -c "mkdir /build && tar -x -C /build && cd /build && packaging/$1/container.sh && cp packaging/out/* /out/"
