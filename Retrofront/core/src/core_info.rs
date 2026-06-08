use std::collections::{BTreeSet, HashMap};
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
    pub database: String,
    pub firmware_count: usize,
}

pub struct CoreInfoList {
    pub cores: Vec<CoreInfo>,
    pub info_dir: PathBuf,
    pub all_extensions: Vec<String>,
}

impl CoreInfoList {
    pub fn new() -> Self {
        Self {
            cores: Vec::new(),
            info_dir: PathBuf::new(),
            all_extensions: Vec::new(),
        }
    }

    pub fn set_info_dir(&mut self, path: PathBuf) {
        self.info_dir = path;
        self.refresh_loaded_info();
    }

    pub fn clear(&mut self) {
        self.cores.clear();
        self.all_extensions.clear();
    }

    pub fn scan_directory(&mut self, cores_dir: &Path) {
        if let Ok(entries) = fs::read_dir(cores_dir) {
            let mut paths: Vec<PathBuf> = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| Self::is_libretro_library(path))
                .collect();
            paths.sort();

            for path in paths {
                if self.cores.iter().any(|c| c.path == path) {
                    continue;
                }
                let mut info = self.load_info_for_core(&path);
                info.path = path;
                self.cores.push(info);
            }
            self.sort_and_resolve_extensions();
        }
    }

    pub fn supported_extensions_for_path(&self, core_path: &Path) -> Vec<String> {
        self.cores
            .iter()
            .find(|core| core.path == core_path)
            .map(|core| core.supported_extensions.clone())
            .unwrap_or_default()
    }

    fn refresh_loaded_info(&mut self) {
        let paths: Vec<PathBuf> = self.cores.iter().map(|core| core.path.clone()).collect();
        self.cores = paths
            .into_iter()
            .map(|path| {
                let mut info = self.load_info_for_core(&path);
                info.path = path;
                info
            })
            .collect();
        self.sort_and_resolve_extensions();
    }

    pub fn rebuild_indexes(&mut self) {
        self.sort_and_resolve_extensions();
    }

    pub fn compatible_cores_for_extension(&self, extension: &str) -> Vec<CoreInfo> {
        let wanted = extension.trim_start_matches('.').to_lowercase();
        if wanted.is_empty() {
            return Vec::new();
        }

        self.cores
            .iter()
            .filter(|core| {
                core.supported_extensions
                    .iter()
                    .any(|ext| ext.trim_start_matches('.').eq_ignore_ascii_case(&wanted))
            })
            .cloned()
            .collect()
    }

    fn sort_and_resolve_extensions(&mut self) {
        self.cores.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
                .then_with(|| a.path.cmp(&b.path))
        });

        let mut all = BTreeSet::new();
        for core in &self.cores {
            for ext in &core.supported_extensions {
                if !ext.is_empty() {
                    all.insert(ext.to_lowercase());
                }
            }
        }
        self.all_extensions = all.into_iter().collect();
    }

    fn is_libretro_library(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        matches!(ext.to_ascii_lowercase().as_str(), "dylib" | "so" | "dll")
    }

    fn load_info_for_core(&self, core_path: &Path) -> CoreInfo {
        let stem = core_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("core");
        let fallback_name = Self::display_name_from_stem(stem);

        let mut info = CoreInfo {
            display_name: fallback_name,
            ..CoreInfo::default()
        };

        let Some(info_path) = self.find_info_path(core_path) else {
            return info;
        };

        if let Ok(content) = fs::read_to_string(info_path) {
            let map = Self::parse_info_file(&content);
            if let Some(val) = map.get("display_name") {
                info.display_name = val.clone();
            }
            if let Some(val) = map.get("display_version") {
                info.display_version = val.clone();
            }
            if let Some(val) = map.get("corename") {
                info.core_name = val.clone();
            }
            if let Some(val) = map.get("system_manufacturer") {
                info.system_manufacturer = val.clone();
            }
            if let Some(val) = map.get("systemname") {
                info.system_name = val.clone();
            }
            if let Some(val) = map.get("supported_extensions") {
                info.supported_extensions = Self::split_list(val);
            }
            if let Some(val) = map.get("authors") {
                info.authors = val.clone();
            }
            if let Some(val) = map.get("permissions") {
                info.permissions = val.clone();
            }
            if let Some(val) = map.get("license") {
                info.licenses = val.clone();
            }
            if let Some(val) = map.get("categories") {
                info.categories = val.clone();
            }
            if let Some(val) = map.get("notes") {
                info.notes = val.clone();
            }
            if let Some(val) = map.get("description") {
                info.description = val.clone();
            }
            if let Some(val) = map.get("database") {
                info.database = val.clone();
            }
            info.firmware_count = map
                .keys()
                .filter(|key| key.starts_with("firmware") && key.ends_with("_path"))
                .count();
        }
        info
    }

    fn find_info_path(&self, core_path: &Path) -> Option<PathBuf> {
        if self.info_dir.as_os_str().is_empty() {
            return None;
        }

        for candidate in Self::info_name_candidates(core_path) {
            let path = self.info_dir.join(format!("{candidate}.info"));
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    fn info_name_candidates(core_path: &Path) -> Vec<String> {
        let Some(stem) = core_path.file_stem().and_then(|stem| stem.to_str()) else {
            return Vec::new();
        };

        let mut candidates = Vec::new();
        Self::push_unique(&mut candidates, stem.to_string());

        let mut normalized = stem.to_string();
        for suffix in [
            "_libretro_ios",
            "_libretro_macos",
            "_libretro_android",
            "_libretro",
            "_ios",
            "_macos",
        ] {
            if let Some(stripped) = normalized.strip_suffix(suffix) {
                normalized = stripped.to_string();
                Self::push_unique(&mut candidates, normalized.clone());
            }
        }

        if let Some(stripped) = stem.strip_prefix("lib") {
            Self::push_unique(&mut candidates, stripped.to_string());
            if let Some(stripped) = stripped.strip_suffix("_libretro") {
                Self::push_unique(&mut candidates, stripped.to_string());
            }
        }

        candidates
    }

    fn parse_info_file(content: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            map.insert(key.trim().to_string(), Self::unquote(value.trim()));
        }
        map
    }

    fn split_list(value: &str) -> Vec<String> {
        value
            .split('|')
            .map(|entry| entry.trim().trim_start_matches('.').to_lowercase())
            .filter(|entry| !entry.is_empty())
            .collect()
    }

    fn unquote(value: &str) -> String {
        let trimmed = value.trim();
        trimmed
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .unwrap_or(trimmed)
            .replace("\\\"", "\"")
    }

    fn display_name_from_stem(stem: &str) -> String {
        Self::info_name_candidates(Path::new(stem))
            .last()
            .cloned()
            .unwrap_or_else(|| stem.to_string())
            .replace('_', " ")
    }

    fn push_unique(values: &mut Vec<String>, value: String) {
        if !value.is_empty() && !values.iter().any(|existing| existing == &value) {
            values.push(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_retroarch_info_candidates_for_ios_libretro_names() {
        let candidates =
            CoreInfoList::info_name_candidates(Path::new("/cores/mgba_libretro_ios.dylib"));
        assert!(candidates.contains(&"mgba_libretro_ios".to_string()));
        assert!(candidates.contains(&"mgba".to_string()));
    }

    #[test]
    fn parses_supported_extensions_like_retroarch_info() {
        let parsed = CoreInfoList::parse_info_file(
            "display_name = \"mGBA\"\nsupported_extensions = \"gba|gb|.gbc\"\n",
        );
        assert_eq!(parsed.get("display_name").unwrap(), "mGBA");
        assert_eq!(
            CoreInfoList::split_list(parsed.get("supported_extensions").unwrap()),
            vec!["gba", "gb", "gbc"]
        );
    }
}
