# RetroFront

RetroFront is an iOS-first SwiftUI frontend for libretro/RetroArch-style emulation. It provides a native library UI, ROM/core import, local metadata, save-state plumbing, and a C libretro host that loads signed dynamic cores through `dlopen`/`dlsym`.

## iOS dynamic core policy

On iOS, runtime executable loading is controlled by code signing and distribution policy. Developer or sideloaded builds can import cores that are already signed for the host app. App Store builds must ship executable cores in the app bundle or through an Apple-approved signed update; the app cannot download and execute arbitrary new code after review.

## Build options

- `swift test` validates the Swift Package targets on platforms where SwiftPM is convenient.
- `xcodegen generate` creates the Xcode project from `project.yml` for native iOS signing, capabilities, and archive workflows.

## Documents layout

- `ROMs/` content files
- `Cores/` signed `.dylib`, `.framework`, or `.so` libretro cores
- `System/` BIOS and firmware files
- `Saves/` SRAM/memory-card files
- `States/` save states
- `Artwork/` local covers and screenshots
- `Shaders/`, `Overlays/`, `Playlists/` frontend assets
