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
        let out_path = destination_dir.join(safe_name);
        if entry.is_dir() || entry.name().ends_with('/') {
            fs::create_dir_all(&out_path).map_err(|e| format!("create {:?}: {e}", out_path))?;
            report.directories_created += 1;
            continue;
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
            safe_zip_path("xmb/monochrome/font.ttf"),
            Some(PathBuf::from("xmb/monochrome/font.ttf"))
        );
    }
}
