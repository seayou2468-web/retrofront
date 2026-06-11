# Retrofront

Retrofront is a libretro frontend foundation for iOS and Linux. The Swift package is pinned to Swift tools version 6.0 so Linux hosts with Swift 6.0.3 can load the manifest without a tools-version mismatch.

The emulator/game cores are libretro cores. Retrofront is a single project: Rust owns the portable frontend runtime and Swift owns the platform UI shells, both built from the same repository root and wired together through a stable C ABI.

## Layout

- `libretro/libretro.h` — canonical libretro C API header.
- `core/` — Rust frontend management core (`retrofront_core`), with bindgen-generated libretro bindings from `libretro/libretro.h`.
- `frontend/CRetrofrontCore/` — C header/module map for Swift ↔ Rust FFI.
- `frontend/Sources/RetrofrontSwift/` — Swift wrapper API for UI code.
- `apps/iOS/Sources/` — iOS SwiftUI app only.
- `apps/linux/Sources/` — Linux Swift Adwaita app only; it is GUI-first and shares the same Swift/Rust runtime as iOS.
- `docs/ARCHITECTURE.md` — architecture and platform notes.

## Build and test

```sh
cd Retrofront
make test
```

For SwiftPM directly, build the Rust library first so Swift can link it:

```sh
cd Retrofront
cargo build --release
swift build
```

If the Linux host does not have GTK/libadwaita development headers installed, build or test the shared Swift/Rust runtime without the GUI target:

```sh
cd Retrofront
RETROFRONT_DISABLE_LINUX_GUI=1 swift test
```


## iOS device app

The iOS app is generated with XcodeGen and builds for real devices only; no iOS simulator setup is required. During the app build, every top-level `*.dylib` and `*.framework` bundle in `cores/` is copied into the app's `Frameworks` directory without linking the app executable against those cores. This keeps libretro cores loadable on demand with `dlopen` instead of causing launch-time `Library not loaded` failures. Bundled cores are discovered automatically from the app bundle and can be selected explicitly before loading a ROM; there is intentionally no user-facing core import flow. The app runs the core frame loop, forwards virtual RetroPad input, and draws software video frames in SwiftUI.

```sh
cd Retrofront
make ios-rust
make xcodegen
make ios-device-build
```

## Linux UI

The Linux target is a Swift Adwaita application, not a CLI smoke test. Install libadwaita development files first (for example `apt install libadwaita-1-dev libgtksourceview-5-dev`), then build and launch the GUI. `make linux-ui` now checks for those packages before SwiftPM starts so a missing GUI stack fails with an actionable message instead of a C module error:

```sh
cd Retrofront
make linux-ui
.build/debug/retrofront-linux
```
