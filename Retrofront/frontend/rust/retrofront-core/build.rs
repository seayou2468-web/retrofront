use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn collect_dirs(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(rel) = path.strip_prefix(root) {
                let rel = rel.to_string_lossy().replace('\\', "/");
                if !rel.is_empty() {
                    out.push(rel);
                }
            }
            collect_dirs(root, &path, out);
        }
    }
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("c")
            || path.extension().and_then(|e| e.to_str()) == Some("h")
        {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

fn main() {
    let header = PathBuf::from("../../libretro/libretro.h");
    let menu_root = PathBuf::from("../../menu");
    println!("cargo:rerun-if-changed={}", header.display());
    println!("cargo:rerun-if-changed={}", menu_root.display());
    println!("cargo:rerun-if-changed=../../../../reference/RetroArch");

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

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("libretro_bindings.rs"))
        .expect("failed to write libretro bindings");

    let reference_root = PathBuf::from("../../../../reference/RetroArch");
    let mut dirs = Vec::new();
    collect_dirs(&reference_root, &reference_root, &mut dirs);
    dirs.sort();
    dirs.dedup();
    let generated = format!(
        "pub const REFERENCE_RETROARCH_DIRS: &[&str] = &[{}];\n",
        dirs.iter()
            .map(|d| format!("\n    {d:?}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    fs::write(out_dir.join("reference_dirs.rs"), generated).expect("write reference dir list");

    let mut menu_sources = Vec::new();
    collect_files(&menu_root, &menu_root, &mut menu_sources);
    menu_sources.sort();
    menu_sources.dedup();
    assert_eq!(
        menu_sources.len(),
        32,
        "Retrofront/frontend/menu must remain the fixed 32-file UI contract"
    );
    let generated = format!(
        "pub const RETROFRONT_MENU_SOURCE_FILES: &[&str] = &[{}];\n",
        menu_sources
            .iter()
            .map(|d| format!("\n    {d:?}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    fs::write(out_dir.join("menu_sources.rs"), generated).expect("write menu source list");
}
