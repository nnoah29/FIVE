#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build
cargo test
python3 tools/check-norms
