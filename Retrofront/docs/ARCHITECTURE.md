# Retrofront architecture

Retrofront separates UI and frontend management:

- **Swift (`frontend/Sources/RetrofrontSwift`)** owns application/UI-facing APIs and is suitable for iOS, macOS, and Linux SwiftUI or CLI frontends.
- **Rust (`core`)** owns portable frontend state: dynamic loading of libretro cores, session lifecycle, game metadata, callback collection, and a stable C ABI.
- **libretro ABI (`libretro/libretro.h`)** remains the canonical header. `core/build.rs` runs bindgen against this header at compile time; Swift talks to Rust through `frontend/CRetrofrontCore/retrofront_core.h`.

## Lifecycle

1. Swift creates `Retrofront`.
2. Swift calls `loadCore(at:)` with a platform-specific libretro dynamic library path.
3. Rust uses `dlopen`/`dlsym`, validates `retro_api_version`, registers callbacks, calls `retro_init`, and caches `retro_get_system_info`.
4. Swift calls `loadGame(at:)` and `runFrame()`.
5. Rust captures video/audio/input/environment callback events so UI layers can decide how to render or play them.

## Platform notes

- Linux uses `.so` libretro cores and links Rust with `libdl`.
- iOS should link the Rust `staticlib` into the Xcode app target. Dynamic loading of third-party cores may be restricted by platform policy, so static or bundled core strategies can be added behind the same Rust management API.


## UI shells

- The iOS SwiftUI app is an empty emulator shell: it connects to the Rust frontend runtime, shows library/play/core/settings screens, and does not require any libretro emulator core to build or launch.
- The Linux UI is a terminal dashboard executable that also connects to the same Swift/Rust runtime without requiring a loaded emulator core.
- iOS project generation uses XcodeGen. Build the Rust `aarch64-apple-ios` static library first, then generate/build the Xcode project for `generic/platform=iOS`.
