#!/usr/bin/env bash
set -euo pipefail
sed -nE '/^\[workspace\.package\]/,/^\[/ s/^version = "([^"]+)"/\1/p' "$(dirname "$0")/../Cargo.toml" | head -1
