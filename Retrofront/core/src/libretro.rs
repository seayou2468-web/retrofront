//! Bindgen-generated Rust declarations for the canonical libretro C ABI.
//!
//! The source of truth is `Retrofront/libretro/libretro.h`; `build.rs` runs
//! bindgen at compile time and writes the Rust declarations into `OUT_DIR`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/libretro_bindings.rs"));
