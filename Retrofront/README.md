# Retrofront

Retrofront is a libretro frontend foundation for iOS and Linux.

The emulator/game cores are libretro cores. The **frontend core** is separate: a Rust management layer that loads libretro cores, manages lifecycle/session state, and exposes a C ABI for Swift UI code.

## Layout

- `libretro/libretro.h` — canonical libretro C API header.
- `core/` — Rust frontend management core (`retrofront_core`), with bindgen-generated libretro bindings from `libretro/libretro.h`.
- `frontend/CRetrofrontCore/` — C header/module map for Swift ↔ Rust FFI.
- `frontend/Sources/RetrofrontSwift/` — Swift wrapper API for UI code.
- `frontend/Sources/retrofront-cli/` — Linux/macOS command-line smoke-test frontend.
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


## iOS device app

The iOS app is generated with XcodeGen and builds for real devices only; no iOS simulator setup is required. The app bundles `dylibs/mgba_libretro_ios.dylib` as its libretro core, loads it at launch, lets the user pick a ROM from Files, runs the core frame loop, forwards virtual RetroPad input, and draws software video frames in SwiftUI.

```sh
cd Retrofront
make ios-rust
make xcodegen
make ios-device-build
```

## Linux UI

```sh
cd Retrofront
make linux-ui
.build/debug/retrofront-linux-ui
```
