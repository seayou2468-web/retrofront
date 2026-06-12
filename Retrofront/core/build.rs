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

    let menu_dir = manifest_dir.join("../menu");
    let menu_header = menu_dir.join("retrofront_menu.h");
    let menu_shim_header = menu_dir.join("retrofront_menu_shim.h");
    println!("cargo:rerun-if-changed={}", menu_header.display());
    println!("cargo:rerun-if-changed={}", menu_shim_header.display());

    let menu_sources = [
        "retrofront_menu.c",
        "menu_driver.c",
        "menu_displaylist.c",
        "menu_setting.c",
        "menu_screensaver.c",
        "menu_contentless_cores.c",
        "menu_explore.c",
        "cbs/menu_cbs_cancel.c",
        "cbs/menu_cbs_deferred_push.c",
        "cbs/menu_cbs_get_value.c",
        "cbs/menu_cbs_info.c",
        "cbs/menu_cbs_label.c",
        "cbs/menu_cbs_left.c",
        "cbs/menu_cbs_ok.c",
        "cbs/menu_cbs_right.c",
        "cbs/menu_cbs_scan.c",
        "cbs/menu_cbs_select.c",
        "cbs/menu_cbs_start.c",
        "cbs/menu_cbs_sublabel.c",
        "cbs/menu_cbs_title.c",
        "drivers/materialui.c",
        "drivers/ozone.c",
        "drivers/rgui.c",
        "drivers/xmb.c",
    ];

    let mut menu_build = cc::Build::new();
    menu_build
        .include(&menu_dir)
        .define("RETROFRONT_MENU_SHIM_ONLY", None)
        .warnings(true);
    for source in menu_sources {
        let path = menu_dir.join(source);
        println!("cargo:rerun-if-changed={}", path.display());
        menu_build.file(path);
    }
    menu_build.compile("retrofront_menu");
}
