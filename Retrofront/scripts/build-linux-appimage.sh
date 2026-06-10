#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPDIR="${ROOT_DIR}/dist/Retrofront.AppDir"
APPIMAGE="${ROOT_DIR}/dist/Retrofront-x86_64.AppImage"
APPIMAGETOOL="${APPIMAGETOOL:-appimagetool}"

cd "${ROOT_DIR}"
make linux-ui

rm -rf "${APPDIR}" "${APPIMAGE}"
mkdir -p \
  "${APPDIR}/usr/bin" \
  "${APPDIR}/usr/lib/retrofront/cores" \
  "${APPDIR}/usr/share/applications" \
  "${APPDIR}/usr/share/icons/hicolor/256x256/apps" \
  "${APPDIR}/usr/share/retrofront/assets"

cp target/release/retrofront "${APPDIR}/usr/bin/retrofront"
if compgen -G "archifacts/linux/*.so" > /dev/null; then
  cp archifacts/linux/*.so "${APPDIR}/usr/lib/retrofront/cores/"
fi
if compgen -G "apps/iOS/Resources/*.zip" > /dev/null; then
  cp apps/iOS/Resources/*.zip "${APPDIR}/usr/share/retrofront/assets/"
fi

cat > "${APPDIR}/AppRun" <<'APPRUN'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
export RETROFRONT_BUNDLED_CORE_DIR="${HERE}/usr/lib/retrofront/cores"
export RETROFRONT_BUNDLED_ASSET_DIR="${HERE}/usr/share/retrofront/assets"
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
print('<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256"><rect width="256" height="256" rx="54" fill="#101820"/><path d="M66 98h124a30 30 0 0 1 29 24l12 58a25 25 0 0 1-42 23l-24-25H91l-24 25a25 25 0 0 1-42-23l12-58a30 30 0 0 1 29-24Z" fill="#2d7ff9"/><path d="M75 131h42M96 110v42" stroke="white" stroke-width="13" stroke-linecap="round"/><circle cx="169" cy="132" r="11" fill="white"/><circle cx="197" cy="155" r="11" fill="white"/></svg>')
PY
cp "${APPDIR}/retrofront.svg" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/retrofront.svg"

if command -v "${APPIMAGETOOL}" >/dev/null 2>&1; then
  ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "${APPIMAGETOOL}" "${APPDIR}" "${APPIMAGE}"
else
  echo "appimagetool not found; leaving AppDir at ${APPDIR}" >&2
  exit 2
fi
