#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_DIR="${RETROFRONT_PROJECT_DIR:-${REPO_DIR}/Retrofront}"
FLUTTER_DIR="${ROOT_DIR}/flutter"
APPDIR="${ROOT_DIR}/dist/Retrofront.AppDir"
APPIMAGE="${ROOT_DIR}/dist/Retrofront-x86_64.AppImage"
APPIMAGETOOL="${APPIMAGETOOL:-appimagetool}"

cd "${ROOT_DIR}"
flutter build linux --release
cargo build --manifest-path Cargo.toml --release -p retrofront-core

rm -rf "${APPDIR}" "${APPIMAGE}"
mkdir -p \
  "${APPDIR}/usr/bin" \
  "${APPDIR}/usr/lib/retrofront/cores" \
  "${APPDIR}/usr/lib/retrofront/native" \
  "${APPDIR}/usr/share/applications" \
  "${APPDIR}/usr/share/icons/hicolor/256x256/apps" \
  "${APPDIR}/usr/share/retrofront/assets"

cp -a "${FLUTTER_DIR}/build/linux/x64/release/bundle/." "${APPDIR}/usr/bin/"
if [[ -f target/release/libretrofront_core.so ]]; then
  cp target/release/libretrofront_core.so "${APPDIR}/usr/lib/retrofront/native/"
fi
if compgen -G "archifacts/linux/*.so" > /dev/null; then
  cp archifacts/linux/*.so "${APPDIR}/usr/lib/retrofront/cores/"
fi
if compgen -G "flutter/assets/retroarch/*.zip" > /dev/null; then
  cp flutter/assets/retroarch/*.zip "${APPDIR}/usr/share/retrofront/assets/"
fi

cat > "${APPDIR}/AppRun" <<'APPRUN'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
export RETROFRONT_BUNDLED_CORE_DIR="${HERE}/usr/lib/retrofront/cores"
export RETROFRONT_BUNDLED_ASSET_DIR="${HERE}/usr/share/retrofront/assets"
export RETROFRONT_BUNDLED_NATIVE_DIR="${HERE}/usr/lib/retrofront/native"
export LD_LIBRARY_PATH="${HERE}/usr/lib/retrofront/native:${HERE}/usr/bin/lib:${LD_LIBRARY_PATH:-}"
exec "${HERE}/usr/bin/retrofront" "$@"
APPRUN
chmod +x "${APPDIR}/AppRun"

cat > "${APPDIR}/retrofront.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Retrofront
Exec=retrofront
Icon=retrofront
Categories=Game;Emulator;
Terminal=false
DESKTOP
cp "${APPDIR}/retrofront.desktop" "${APPDIR}/usr/share/applications/retrofront.desktop"

python3 - <<'PY' > "${APPDIR}/retrofront.svg"
print('<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256"><rect width="256" height="256" rx="54" fill="#050b14"/><circle cx="86" cy="82" r="30" fill="#7b61ff"/><circle cx="112" cy="110" r="40" fill="#1fd7f5" opacity=".35"/><path d="M50 150h156a28 28 0 0 1 27 21l8 31a20 20 0 0 1-34 18l-24-25H73l-24 25a20 20 0 0 1-34-18l8-31a28 28 0 0 1 27-21Z" fill="#111d2f" stroke="#7b61ff" stroke-width="6"/><path d="M75 176h42M96 155v42" stroke="white" stroke-width="10" stroke-linecap="round"/><circle cx="171" cy="178" r="10" fill="white"/><circle cx="198" cy="196" r="10" fill="white"/></svg>')
PY
cp "${APPDIR}/retrofront.svg" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/retrofront.svg"

if command -v "${APPIMAGETOOL}" >/dev/null 2>&1; then
  ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "${APPIMAGETOOL}" "${APPDIR}" "${APPIMAGE}"
else
  echo "appimagetool not found; leaving AppDir at ${APPDIR}" >&2
  exit 2
fi
