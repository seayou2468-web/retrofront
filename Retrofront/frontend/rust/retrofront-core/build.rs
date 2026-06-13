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

fn main() {
    let header = PathBuf::from("../../libretro/libretro.h");
    println!("cargo:rerun-if-changed={}", header.display());
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
}
