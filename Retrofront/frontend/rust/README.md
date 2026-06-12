# Retrofront Rust backend

This directory contains the Rust-owned backend for the fixed C menu sources in
`Retrofront/frontend/menu`.

## Contract

The C menu is treated as the specification layer.  Its dependencies are provided
by Rust instead of RetroArch globals:

| Menu dependency | Rust module | Notes |
| --- | --- | --- |
| video drawing / renderer | `retrofront_core::renderer` | Uses `wgpu` on Linux and physical iOS targets. |
| shader pipeline | `retrofront_core::shader` | Integrates ordinary `librashader` Vulkan/Metal runtimes from raw handles extracted from `wgpu`; the `librashader` wgpu runtime is intentionally not enabled. |
| input system | `retrofront_core::input` | Platform key/gamepad/touch events are normalized to menu actions. |
| filesystem | `retrofront_core::fs` | Owns config, playlists, shaders, saves and states directories. |
| settings | `retrofront_core::settings` | JSON-backed typed setting store. |
| task queue | `retrofront_core::task` | Rust task runner with pollable completions for menu refresh. |
| playlists | `retrofront_core::playlist` | JSON playlist load/save/list implementation. |
| libretro core loading | `retrofront_core::core` and `retrofront_core::libretro` | `libretro.h` is generated with bindgen, then wrapped in safe Rust structs before use. |

## C bridge

`retrofront-core/cinclude/retrofront_rust.h` exposes the initial thin C ABI.  Menu
callbacks should call this ABI and avoid depending on renderer/input/filesystem
implementation details.

## iOS note

The common core loader uses the same dynamic-library path for Linux and real iOS
devices.  Simulator-specific code paths are intentionally absent.
