# retrofront

Rust + Swift libretro frontend foundation for Linux and real iOS devices.

## Build

```bash
cargo test --workspace
(cd swift && swift test)
```

## Linux run

```bash
cargo run -p retrofront-cli -- /path/to/core_libretro.so /path/to/game.rom 60
RETROFRONT_BUTTONS="START=1,A=1" cargo run -p retrofront-cli -- /path/to/core_libretro.so /path/to/game.rom 1
```

## iOS policy

The Swift frontend rejects iOS Simulator builds. Device builds should link the Rust `staticlib` produced by `retrofront-core` and call the C ABI in `crates/retrofront-core/src/ffi.rs`.

## Controller skins

Controller skins are parsed by Rust and mirrored in Swift. A skin file can define hit zones like `button.a=A,0,circle,0.86,0.72,0.055` or `button.start=START,0,rect,0.53,0.84,0.10,0.05`; touches map directly to libretro joypad IDs.
