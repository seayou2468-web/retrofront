# RetroFront

RetroFront is an iOS-first SwiftUI frontend for libretro/RetroArch-style emulation. It provides a native library UI, ROM/core import, local metadata, save-state plumbing, soft-patch handling, touch and MFi controller input, artwork, BIOS auditing, and a C libretro host that loads signed dynamic cores through `dlopen`/`dlsym`.

## Implemented frontend feature set

The app targets the feature set expected from a modern multi-emulator frontend: searchable playlists/library shelves, per-system browsing, artwork matching, signed core management, save states, rewind buffering, fast-forward, run-ahead settings, hardware and touch controls, cheat persistence, IPS soft patches, shader/overlay presets, BIOS auditing, RetroAchievements account storage, netplay room settings, and dynamic-core policy reporting.

## iOS dynamic core policy

On iOS, runtime executable loading is controlled by code signing and distribution policy. Developer or sideloaded builds can import cores that are already signed for the host app. App Store builds must ship executable cores in the app bundle or through an Apple-approved signed update; the app cannot download and execute arbitrary new code after review.

## Build options

- `swift test` validates the Swift Package targets on platforms where SwiftPM is convenient.
- `xcodegen generate` creates the Xcode project from `project.yml` for native iOS signing, capabilities, and archive workflows.
- `.github/workflows/ios-device.yml` archives for generic physical iOS devices on GitHub-hosted macOS and includes a separate self-hosted real-device test job (`self-hosted`, `macOS`, `iOS-device`) that never targets the simulator.

## Documents layout

- `ROMs/` content files
- `Cores/` signed `.dylib`, `.framework`, or `.so` libretro cores
- `System/` BIOS and firmware files
- `Saves/` SRAM/memory-card files
- `States/` save states
- `Artwork/` local covers and screenshots
- `Shaders/`, `Overlays/`, `Playlists/` frontend assets
- `Import Inbox/` temporary patched content and imported assets
