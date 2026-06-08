use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct CoreInfo {
    pub path: PathBuf,
    pub display_name: String,
    pub display_version: String,
    pub core_name: String,
    pub system_manufacturer: String,
    pub system_name: String,
    pub supported_extensions: Vec<String>,
    pub authors: String,
    pub permissions: String,
    pub licenses: String,
    pub categories: String,
    pub notes: String,
    pub description: String,
}

pub struct CoreInfoList {
    pub cores: Vec<CoreInfo>,
    pub info_dir: PathBuf,
}

impl CoreInfoList {
    pub fn new() -> Self {
        Self {
            cores: Vec::new(),
            info_dir: PathBuf::new(),
        }
    }

    pub fn set_info_dir(&mut self, path: PathBuf) {
        self.info_dir = path;
    }

    pub fn clear(&mut self) {
        self.cores.clear();
    }

    pub fn scan_directory(&mut self, cores_dir: &Path) {
        if let Ok(entries) = fs::read_dir(cores_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_lib = if cfg!(target_os = "windows") {
                    path.extension().map_or(false, |e| e == "dll")
                } else if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
                    path.extension().map_or(false, |e| e == "dylib")
                } else {
                    path.extension().map_or(false, |e| e == "so")
                };

                if is_lib {
                    // Check if already present
                    if self.cores.iter().any(|c| c.path == path) {
                        continue;
                    }
                    let mut info = self.load_info_for_core(&path);
                    info.path = path;
                    self.cores.push(info);
                }
            }
        }
    }

    fn load_info_for_core(&self, core_path: &Path) -> CoreInfo {
        let stem = core_path.file_stem().unwrap().to_string_lossy();
        // RetroArch often removes _libretro and platform suffix
        let info_stem = stem.replace("_libretro", "").replace("_ios", "");
        let info_path = self.info_dir.join(format!("{}.info", info_stem));

        let mut info = CoreInfo::default();
        info.display_name = info_stem.clone();

        if let Ok(content) = fs::read_to_string(&info_path) {
            let mut map = HashMap::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim().trim_matches('"');
                    map.insert(key, value.to_string());
                }
            }

            if let Some(val) = map.get("display_name") { info.display_name = val.clone(); }
            if let Some(val) = map.get("display_version") { info.display_version = val.clone(); }
            if let Some(val) = map.get("corename") { info.core_name = val.clone(); }
            if let Some(val) = map.get("system_manufacturer") { info.system_manufacturer = val.clone(); }
            if let Some(val) = map.get("systemname") { info.system_name = val.clone(); }
            if let Some(val) = map.get("supported_extensions") {
                info.supported_extensions = val.split('|').map(|s| s.to_string()).collect();
            }
            if let Some(val) = map.get("authors") { info.authors = val.clone(); }
            if let Some(val) = map.get("permissions") { info.permissions = val.clone(); }
            if let Some(val) = map.get("license") { info.licenses = val.clone(); }
            if let Some(val) = map.get("categories") { info.categories = val.clone(); }
            if let Some(val) = map.get("notes") { info.notes = val.clone(); }
            if let Some(val) = map.get("description") { info.description = val.clone(); }
        }
        info
    }
}
