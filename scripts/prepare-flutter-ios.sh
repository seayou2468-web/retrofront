#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_DIR="${RETROFRONT_PROJECT_DIR:-${REPO_DIR}/Retrofront}"
FLUTTER_DIR="${ROOT_DIR}/flutter"
FLUTTER_BIN="${FLUTTER:-flutter}"

cd "${FLUTTER_DIR}"
if [[ ! -d ios ]]; then
  "${FLUTTER_BIN}" create --platforms=ios -i objc .
fi

if find ios -name '*.swift' -print | grep -q .; then
  echo "error: Swift sources found in ${FLUTTER_DIR}/ios. Recreate the Flutter iOS shell with: flutter create --platforms=ios -i objc ." >&2
  exit 1
fi

"${FLUTTER_BIN}" pub get

PODFILE="${FLUTTER_DIR}/ios/Podfile"
if [[ -f "${PODFILE}" ]] && ! grep -q 'RETROFRONT_NO_SWIFT_STDLIB' "${PODFILE}"; then
  python3 - "${PODFILE}" <<'PYTHON'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
settings = """\
      # RETROFRONT_NO_SWIFT_STDLIB: keep the generated iOS shell and local
      # bridge plugin Objective-C-only so libswift*.dylib is not embedded.
      config.build_settings['ALWAYS_EMBED_SWIFT_STANDARD_LIBRARIES'] = 'NO'
      config.build_settings['SWIFT_VERSION'] = '' unless target.source_build_phase.files.any? { |f| f.file_ref && f.file_ref.path.to_s.end_with?('.swift') }
"""
needle = '      flutter_additional_ios_build_settings(target)\n'
if needle in text:
    text = text.replace(
        needle,
        needle + "      target.build_configurations.each do |config|\n" + settings + "      end\n",
        1,
    )
else:
    text += """

post_install do |installer|
  installer.pods_project.targets.each do |target|
    flutter_additional_ios_build_settings(target) if defined?(flutter_additional_ios_build_settings)
    target.build_configurations.each do |config|
      # RETROFRONT_NO_SWIFT_STDLIB: keep the generated iOS shell and local
      # bridge plugin Objective-C-only so libswift*.dylib is not embedded.
      config.build_settings['ALWAYS_EMBED_SWIFT_STANDARD_LIBRARIES'] = 'NO'
      config.build_settings['SWIFT_VERSION'] = '' unless target.source_build_phase.files.any? { |f| f.file_ref && f.file_ref.path.to_s.end_with?('.swift') }
    end
  end
end
"""
path.write_text(text)
PYTHON
fi
