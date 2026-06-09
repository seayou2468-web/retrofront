use std::fs;
use std::io;
use std::path::Path;
use zip::ZipArchive;

pub fn extract_assets_zip(zip_path: &Path, target_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("Failed to open assets.zip: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to parse zip: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Failed to read zip entry: {e}"))?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| format!("Failed to create directory: {e}"))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| format!("Failed to create parent directory: {e}"))?;
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| format!("Failed to create file: {e}"))?;
            io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to copy file: {e}"))?;
        }
    }
    Ok(())
}
