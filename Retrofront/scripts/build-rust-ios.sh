#!/usr/bin/env bash
set -euo pipefail

TARGET="${IOS_RUST_TARGET:-aarch64-apple-ios}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

rustup target add "$TARGET"
cargo build --manifest-path "$ROOT_DIR/Cargo.toml" --release -p retrofront-ui --lib --target "$TARGET"

LIB="$ROOT_DIR/target/$TARGET/release/libretrofront_ui.a"
if [[ ! -f "$LIB" ]]; then
  echo "Missing expected Rust Slint UI static library: $LIB" >&2
  exit 1
fi

echo "Built $LIB"
