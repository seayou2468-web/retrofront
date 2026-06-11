use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let header = manifest_dir.join("../libretro/libretro.h");

    println!("cargo:rerun-if-changed={}", header.display());

    let clang_target = match env::var("TARGET").as_deref() {
        Ok("aarch64-apple-ios") => Some("arm64-apple-ios"),
        Ok("aarch64-apple-ios-sim") => Some("arm64-apple-ios-simulator"),
        Ok("x86_64-apple-ios") => Some("x86_64-apple-ios-simulator"),
        _ => None,
    };

    let mut builder = bindgen::Builder::default().header(header.to_string_lossy());
    if let Some(clang_target) = clang_target {
        builder = builder.clang_arg(format!("--target={clang_target}"));

        if let Ok(sdkroot) = env::var("SDKROOT") {
            if !sdkroot.is_empty() {
                builder = builder.clang_arg("-isysroot").clang_arg(sdkroot);
            }
        }

        if let Ok(deployment_target) = env::var("IPHONEOS_DEPLOYMENT_TARGET") {
            if !deployment_target.is_empty() {
                builder = builder.clang_arg(format!("-miphoneos-version-min={deployment_target}"));
            }
        }
    }

    let bindings = builder
        .allowlist_type("retro_.*")
        .allowlist_function("retro_.*")
        .allowlist_var("RETRO_.*")
        .derive_debug(true)
        .derive_default(true)
        .layout_tests(false)
        .generate_comments(false)
        .generate()
        .expect("failed to generate libretro bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    bindings
        .write_to_file(out_path.join("libretro_bindings.rs"))
        .expect("failed to write libretro bindings");
}
