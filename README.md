# retrofront

Retrofront is a Flutter + Rust libretro frontend targeting iOS and Linux.

## Layout and UI

The previous Slint shell has been removed from the Cargo workspace. Flutter now
owns all application UI from `Retrofront/flutter/lib/main.dart`, with separate
mobile and desktop compositions so iOS never receives Linux-only layout chrome
and Linux never receives the compact mobile tab shell.

## Native core

Rust code is consolidated into one Cargo package, `retrofront-core`, under
`Retrofront/core`. Flutter calls the existing C ABI with `dart:ffi` for core
loading, ROM launch, run-frame, save states, joypad input, overlays, core
options, settings, and frame copy.

## Commands

```bash
cd Retrofront
cargo test --manifest-path Cargo.toml -p retrofront-core
cd flutter && flutter test
```

```bash
cd Retrofront
make flutter-linux
make flutter-ios
```
