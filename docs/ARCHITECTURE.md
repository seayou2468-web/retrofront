# RetroFront architecture

RetroFront is split into a Rust libretro host and Swift platform frontends.

- `retrofront-core` owns the libretro ABI, dynamic core loading on Unix, callbacks, configuration, save/system paths, frame execution, controller-skin hit testing and C ABI functions for Swift.
- `retrofront-cli` is the Linux command-line frontend that can load a `*_libretro.so`, load content and execute frames.
- `swift/RetroFrontKit` contains the shared Swift menu/runtime/controller-skin model for Linux and real iOS devices.
- iOS Simulator is intentionally rejected in Swift code; production iOS integration should link the Rust `staticlib` into a device-only app target.

This repository also keeps the original RetroArch source under `reference/` for compatibility audits while the Rust/Swift implementation is developed.
