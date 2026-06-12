use std::{env, path::PathBuf};

fn main() {
    let header = PathBuf::from("../../libretro/libretro.h");
    println!("cargo:rerun-if-changed={}", header.display());

    let bindings = bindgen::Builder::default()
        .header(header.display().to_string())
        .allowlist_type("retro_.*")
        .allowlist_function("retro_.*")
        .allowlist_var("RETRO_.*")
        .derive_default(true)
        .layout_tests(false)
        .generate_comments(false)
        .generate()
        .expect("failed to generate libretro.h bindings with bindgen");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("libretro_bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("failed to write libretro bindings");
}
