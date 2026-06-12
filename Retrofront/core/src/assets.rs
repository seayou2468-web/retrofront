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
        let safe_name = strip_matching_destination_root(&safe_name, destination_dir);
        if safe_name.as_os_str().is_empty() {
            continue;
        }
        if is_macos_metadata_path(&safe_name) {
            continue;
        }
        let out_path = destination_dir.join(&safe_name);
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

fn strip_matching_destination_root(path: &Path, destination_dir: &Path) -> PathBuf {
    let Some(destination_name) = destination_dir.file_name() else {
        return path.to_path_buf();
    };
    let mut components = path.components();
    let Some(Component::Normal(first)) = components.next() else {
        return path.to_path_buf();
    };
    if first == destination_name {
        components.as_path().to_path_buf()
    } else {
        path.to_path_buf()
    }
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
    fn strips_redundant_archive_root_matching_destination() {
        assert_eq!(
            strip_matching_destination_root(
                Path::new("assets/ozone/png/retroarch.png"),
                Path::new("/tmp/RetroArch/assets")
            ),
            PathBuf::from("ozone/png/retroarch.png")
        );
        assert_eq!(
            strip_matching_destination_root(
                Path::new("overlays/gamepads/retropad.cfg"),
                Path::new("/tmp/RetroArch/overlays")
            ),
            PathBuf::from("gamepads/retropad.cfg")
        );
        assert_eq!(
            strip_matching_destination_root(
                Path::new("glui/add.png"),
                Path::new("/tmp/RetroArch/assets")
            ),
            PathBuf::from("glui/add.png")
        );
    }

    #[test]
    fn preserves_retroarch_frontend_zip_relative_paths() {
        let assets_destination = Path::new("/tmp/RetroArch/assets");
        let info_destination = Path::new("/tmp/RetroArch/info");
        let overlays_destination = Path::new("/tmp/RetroArch/overlays");
        assert_eq!(
            assets_destination.join(safe_zip_path("ozone/png/retroarch.png").unwrap()),
            PathBuf::from("/tmp/RetroArch/assets/ozone/png/retroarch.png")
        );
        assert_eq!(
            info_destination.join(safe_zip_path("mgba_libretro.info").unwrap()),
            PathBuf::from("/tmp/RetroArch/info/mgba_libretro.info")
        );
        assert_eq!(
            overlays_destination.join(safe_zip_path("gamepads/flat/gba.cfg").unwrap()),
            PathBuf::from("/tmp/RetroArch/overlays/gamepads/flat/gba.cfg")
        );
        assert!(is_macos_metadata_path(Path::new(
            "__MACOSX/assets/info/._mgba_libretro.info"
        )));
    }
}
