use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let header = manifest_dir.join("../libretro/libretro.h");

    println!("cargo:rerun-if-changed={}", header.display());
    println!("cargo:rerun-if-env-changed=SDKROOT");

    let target = env::var("TARGET").unwrap_or_default();
    let clang_target = match target.as_str() {
        "aarch64-apple-ios" => Some(("arm64-apple-ios", "iphoneos")),
        "aarch64-apple-ios-sim" => Some(("arm64-apple-ios-simulator", "iphonesimulator")),
        "x86_64-apple-ios" => Some(("x86_64-apple-ios-simulator", "iphonesimulator")),
        _ => None,
    };

    let mut builder = bindgen::Builder::default().header(header.to_string_lossy());
    if let Some((clang_target, sdk_name)) = clang_target {
        if let Some(sdkroot) = apple_sdkroot(sdk_name) {
            builder = builder
                .clang_arg(format!("--target={clang_target}"))
                .clang_arg("-isysroot")
                .clang_arg(sdkroot.to_string_lossy());
        } else {
            println!(
                "cargo:warning=Apple target {target} requested without an iOS SDK; generating target-agnostic libretro bindings for non-macOS checks"
            );
            builder = builder
                .clang_arg("--target=x86_64-linux-gnu")
                .clang_arg("-D__x86_64__")
                .clang_arg("-D__LP64__");
            if let Some(include_dir) = host_multiarch_include() {
                builder = builder
                    .clang_arg("-isystem")
                    .clang_arg(include_dir.to_string_lossy());
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

fn apple_sdkroot(sdk_name: &str) -> Option<PathBuf> {
    env::var_os("SDKROOT")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            let output = Command::new("xcrun")
                .args(["--sdk", sdk_name, "--show-sdk-path"])
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let sdkroot = String::from_utf8(output.stdout).ok()?;
            let sdkroot = sdkroot.trim();
            (!sdkroot.is_empty()).then(|| PathBuf::from(sdkroot))
        })
}

fn host_multiarch_include() -> Option<PathBuf> {
    for candidate in [
        "/usr/include/x86_64-linux-gnu",
        "/usr/include/aarch64-linux-gnu",
    ] {
        let path = PathBuf::from(candidate);
        if path.join("bits/libc-header-start.h").exists() {
            return Some(path);
        }
    }
    None
}
