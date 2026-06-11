#!/usr/bin/env bash
set -euo pipefail

TARGET="${IOS_RUST_TARGET:-aarch64-apple-ios}"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_DIR="${RETROFRONT_PROJECT_DIR:-${REPO_DIR}/Retrofront}"
PROFILE_DIR="$ROOT_DIR/target/$TARGET/release"

rustup target add "$TARGET"
RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-Wl,-dead_strip" \
cargo build \
  --manifest-path "$ROOT_DIR/Cargo.toml" \
  --release \
  -p retrofront-core \
  --target "$TARGET"

CORE_LIB="$PROFILE_DIR/libretrofront_core.a"
if [[ ! -f "$CORE_LIB" ]]; then
  echo "Missing expected Rust FFI static library: $CORE_LIB" >&2
  exit 1
fi

STRIP_TOOL="${IOS_STRIP_TOOL:-}"
if [[ -z "$STRIP_TOOL" ]]; then
  for candidate in llvm-strip xcrun; do
    if command -v "$candidate" >/dev/null 2>&1; then
      STRIP_TOOL="$candidate"
      break
    fi
  done
fi
if [[ -n "$STRIP_TOOL" ]]; then
  if [[ "$(basename "$STRIP_TOOL")" == "xcrun" ]]; then
    xcrun strip -S "$CORE_LIB" 2>/dev/null || true
  else
    "$STRIP_TOOL" -S "$CORE_LIB" 2>/dev/null || true
  fi
fi

printf 'Built %s (%s)\n' "$CORE_LIB" "$(du -h "$CORE_LIB" | awk '{print $1}')"
printf 'Largest iOS core artifacts (kept, not filtered):\n'
if [[ -d "$ROOT_DIR/archifacts/ios" ]]; then
  find "$ROOT_DIR/archifacts/ios" -maxdepth 1 \( -name '*.dylib' -o -name '*.framework' \) -print0 \
    | xargs -0 du -sh 2>/dev/null \
    | sort -hr \
    | head -10
else
  printf '  no archifacts/ios directory\n'
fi
