# Retrofront architecture

Retrofront is one project with platform-separated apps and shared frontend management:

- **Swift (`frontend/Sources/RetrofrontSwift`)** owns shared application/UI-facing APIs, including the RetroArch-compatible storage layout used by both apps.
- **Rust (`core`)** owns portable frontend state: dynamic loading of libretro cores, session lifecycle, game metadata, callback collection, and a stable C ABI.
- **libretro ABI (`libretro/libretro.h`)** remains the canonical header. `core/build.rs` runs bindgen against this header at compile time; Swift talks to Rust through `frontend/CRetrofrontCore/retrofront_core.h`.

## Lifecycle

1. Swift creates `Retrofront`.
2. Swift calls `loadCore(at:)` with a platform-specific libretro dynamic library path.
3. Rust uses `dlopen`/`dlsym`, validates `retro_api_version`, registers callbacks, calls `retro_init`, and caches `retro_get_system_info`.
4. Swift calls `loadGame(at:)` and `runFrame()`.
5. Rust captures video/audio/input/environment callback events and copies the core-provided video buffer into a shared `gfx` frame store.
6. iOS/Linux UI buttons call back into Swift, which calls Rust (`runFrame`, `setGfxBackend`, and frame-copy APIs) so each emulation core is started and drawn through the same runtime path.

## Rust gfx path

- `core/src/gfx.rs` implements the portable software-frame path first: `RETRO_ENVIRONMENT_SET_PIXEL_FORMAT` selects 0RGB1555, RGB565, or XRGB8888, and every `retro_video_refresh_t` buffer is copied and normalized to tight RGBA8888.
- The C/Swift ABI exposes `rf_frontend_video_frame_info` and `rf_frontend_copy_video_frame_rgba`, allowing Linux and iOS to upload the latest Rust-owned RGBA frame to their native surfaces.
- Backend selection is shared (`software`, `metal`, `moltenvk`, `opengl`). iOS stores the selected route in settings, maps software to the CPU path, and maps Metal/MoltenVK/OpenGL ES to the wgpu host path while keeping core launch, frame ingestion, and format conversion in Rust.
- Hardware-render requests from libretro cores are captured from `RETRO_ENVIRONMENT_SET_HW_RENDER` so platform OpenGL/Vulkan surface integration can use the same Rust state instead of duplicating frontend logic.

## Platform notes

- Linux uses `.so` libretro cores and links Rust with `libdl`.
- iOS should link the Rust `staticlib` into the Xcode app target. Dynamic loading of third-party cores may be restricted by platform policy, so static or bundled core strategies can be added behind the same Rust management API.


## UI shells

- The iOS SwiftUI app is an empty emulator shell: it connects to the Rust frontend runtime, shows library/play/core/settings screens, and does not require any libretro emulator core to build or launch.
- The Linux UI lives under `apps/linux`, imports Swift Adwaita through SwiftPM, and starts as a GUI dashboard rather than a CLI. It connects to the same Swift/Rust runtime without requiring a loaded emulator core.
- iOS project generation uses XcodeGen. Build the Rust `aarch64-apple-ios` static library first, then generate/build the Xcode project for `generic/platform=iOS`.

## Platform source separation

- iOS-only SwiftUI code lives in `apps/iOS/Sources`.
- Linux-only Swift Adwaita code lives in `apps/linux/Sources`; it no longer imports a local GTK C shim.
- Shared Swift code lives in `frontend/Sources/RetrofrontSwift`; platform apps must not duplicate storage layout or runtime wrapper code that can live there.
