# Retrofront Flutter UI

`lib/main.dart` is the single entry point for both iOS and Linux. The app uses a
strict platform split at the layout boundary:

- iOS and narrow windows render the mobile tab layout.
- Linux desktop windows render the PC dashboard layout.

The UI talks to the Rust `retrofront-core` C ABI through `dart:ffi` and falls
back to a deterministic demo runtime when the native library is unavailable, so
widget tests can run without bundled libretro cores.

## iOS build/runtime notes

Run `make flutter-ios-prepare` (or any iOS make target) before opening the iOS
project. The prepare step creates an Objective-C Flutter shell, runs `flutter
pub get`, and patches the generated Podfile so Objective-C-only pods do not
embed `libswift*.dylib`.

The iOS document picker is provided by the local `retrofront_ios_bridge` Flutter
plugin. It registers the `retrofront/document_picker` channel through CocoaPods,
so the file picker works in normal `flutter build ios` flows instead of relying
on an XcodeGen-only source file. Picked files are copied into a temporary app
sandbox import directory before their paths are returned to Dart; this avoids
security-scoped provider URLs becoming unreadable during ROM import.

`Core load failed: native core library is unavailable.` means the Rust FFI
library was not found in the app bundle. Build it with `make ios-rust` and make
sure `libretrofront_core.dylib` is embedded in `Runner.app/Frameworks` (or
available via `@rpath`) before launching the app.
