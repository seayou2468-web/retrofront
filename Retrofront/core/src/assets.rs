use std::fs::{self, File};
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetInstallReport {
    pub files_written: usize,
    pub directories_created: usize,
}

pub fn install_assets_zip(
    zip_path: &Path,
    destination_dir: &Path,
) -> Result<AssetInstallReport, String> {
    let file = File::open(zip_path).map_err(|e| format!("open assets.zip: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("read assets.zip: {e}"))?;
    fs::create_dir_all(destination_dir).map_err(|e| format!("create assets directory: {e}"))?;

    let mut report = AssetInstallReport {
        files_written: 0,
        directories_created: 0,
    };
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("read zip entry {i}: {e}"))?;
        let Some(safe_name) = safe_zip_path(entry.name()) else {
            continue;
        };
        if is_macos_metadata_path(&safe_name) {
            continue;
        }
        let out_path = destination_dir.join(normalize_frontend_asset_zip_path(&safe_name));
        if entry.is_dir() || entry.name().ends_with('/') {
            if out_path.is_file() {
                fs::remove_file(&out_path).map_err(|e| format!("replace {:?}: {e}", out_path))?;
            }
            fs::create_dir_all(&out_path).map_err(|e| format!("create {:?}: {e}", out_path))?;
            report.directories_created += 1;
            continue;
        }
        if out_path.is_dir() {
            fs::remove_dir_all(&out_path).map_err(|e| format!("replace {:?}: {e}", out_path))?;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {:?}: {e}", parent))?;
        }
        let mut out = File::create(&out_path).map_err(|e| format!("create {:?}: {e}", out_path))?;
        io::copy(&mut entry, &mut out).map_err(|e| format!("extract {:?}: {e}", out_path))?;
        report.files_written += 1;
    }
    Ok(report)
}

fn normalize_frontend_asset_zip_path(path: &Path) -> PathBuf {
    let mut components = path.components();
    let Some(Component::Normal(first)) = components.clone().next() else {
        return path.to_path_buf();
    };
    if first == "assets" {
        while matches!(components.clone().next(), Some(Component::Normal(part)) if part == "assets") {
            components.next();
        }
        return components.as_path().to_path_buf();
    }
    if first == "info" || first == "overlays" {
        components.next();
        return components.as_path().to_path_buf();
    }
    path.to_path_buf()
}

fn is_macos_metadata_path(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(component, Component::Normal(part) if part == "__MACOSX" || part.to_string_lossy().starts_with("._"))
    })
}

fn safe_zip_path(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if safe.as_os_str().is_empty() {
        None
    } else {
        Some(safe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zip_slip_paths() {
        assert_eq!(safe_zip_path("../escape"), None);
        assert_eq!(safe_zip_path("/absolute"), None);
        assert_eq!(safe_zip_path("x/../../escape"), None);
        assert_eq!(
            safe_zip_path("oneui/dark/font.ttf"),
            Some(PathBuf::from("oneui/dark/font.ttf"))
        );
    }

    #[test]
    fn normalizes_retroarch_frontend_asset_zip_roots() {
        assert_eq!(
            normalize_frontend_asset_zip_path(Path::new("assets/info/mgba_libretro.info")),
            PathBuf::from("info/mgba_libretro.info")
        );
        assert_eq!(
            normalize_frontend_asset_zip_path(Path::new("assets/assets/info/azahar_libretro.info")),
            PathBuf::from("info/azahar_libretro.info")
        );
        assert_eq!(
            normalize_frontend_asset_zip_path(Path::new("info/mgba_libretro.info")),
            PathBuf::from("mgba_libretro.info")
        );
        assert_eq!(
            normalize_frontend_asset_zip_path(Path::new("overlays/gamepads/flat/retropad.cfg")),
            PathBuf::from("gamepads/flat/retropad.cfg")
        );
        assert_eq!(
            normalize_frontend_asset_zip_path(Path::new("xmb/monochrome/font.ttf")),
            PathBuf::from("xmb/monochrome/font.ttf")
        );
        assert!(is_macos_metadata_path(Path::new(
            "__MACOSX/assets/info/._mgba_libretro.info"
        )));
    }
}
