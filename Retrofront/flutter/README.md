# Retrofront Flutter UI

`lib/main.dart` is the single entry point for both iOS and Linux. The app uses a
strict platform split at the layout boundary:

- iOS and narrow windows render the mobile tab layout.
- Linux desktop windows render the PC dashboard layout.

The UI talks to the Rust `retrofront-core` C ABI through `dart:ffi` and falls
back to a deterministic demo runtime when the native library is unavailable, so
widget tests can run without bundled libretro cores.
