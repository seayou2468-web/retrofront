#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${RETROFRONT_FRONTEND_ASSET_URL:-https://buildbot.libretro.com/assets/frontend}"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_DIR="${RETROFRONT_PROJECT_DIR:-${REPO_DIR}/Retrofront}"
DEST_DIR="${1:-${ROOT_DIR}/apps/iOS/Resources}"
PACKAGES=(assets info overlays)

mkdir -p "${DEST_DIR}"
for package in "${PACKAGES[@]}"; do
  url="${BASE_URL}/${package}.zip"
  dest="${DEST_DIR}/${package}.zip"
  tmp="${dest}.tmp"
  echo "Fetching ${url} -> ${dest}"
  curl --fail --location --retry 3 --retry-delay 2 --output "${tmp}" "${url}"
  mv "${tmp}" "${dest}"
done
