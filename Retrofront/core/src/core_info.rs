use std::collections::{BTreeSet, HashMap};
use std::ffi::{CStr, CString};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

use crate::dylib::Library;
use crate::libretro;

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
        let mut paths = Vec::new();
        Self::collect_core_paths(cores_dir, &mut paths, 0);
        paths.sort();
        paths.dedup();

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

    pub fn core_for_path(&self, core_path: &Path) -> Option<CoreInfo> {
        if let Some(core) = self
            .cores
            .iter()
            .find(|core| paths_equal(&core.path, core_path))
            .cloned()
        {
            return Some(core);
        }
        if !core_path.exists() {
            return None;
        }
        let mut info = self.load_info_for_core(core_path);
        info.path = core_path.to_path_buf();
        Some(info)
    }

    pub fn register_core_path(&mut self, core_path: &Path) -> bool {
        let Some(info) = self.core_for_path(core_path) else {
            return false;
        };
        if !self
            .cores
            .iter()
            .any(|core| paths_equal(&core.path, core_path))
        {
            self.cores.push(info);
            self.sort_and_resolve_extensions();
        }
        true
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
        self.compatible_cores_for_extensions([wanted])
    }

    pub fn compatible_cores_for_content_path(&self, content_path: &Path) -> Vec<CoreInfo> {
        let mut wanted = Vec::new();
        if let Some(ext) = content_path.extension().and_then(|ext| ext.to_str()) {
            let ext = ext.trim_start_matches('.').to_lowercase();
            if !ext.is_empty() {
                wanted.push(ext);
            }
        }

        if content_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            wanted.extend(zip_member_extensions(content_path));
        }

        wanted.sort();
        wanted.dedup();
        self.compatible_cores_for_extensions(wanted)
    }

    fn compatible_cores_for_extensions<I>(&self, extensions: I) -> Vec<CoreInfo>
    where
        I: IntoIterator<Item = String>,
    {
        let wanted: Vec<String> = extensions
            .into_iter()
            .map(|ext| ext.trim_start_matches('.').to_lowercase())
            .filter(|ext| !ext.is_empty())
            .collect();
        if wanted.is_empty() {
            return Vec::new();
        }

        self.cores
            .iter()
            .filter(|core| {
                wanted.iter().any(|wanted_ext| {
                    core.supported_extensions
                        .iter()
                        .any(|ext| ext.trim_start_matches('.').eq_ignore_ascii_case(wanted_ext))
                })
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

    fn collect_core_paths(dir: &Path, paths: &mut Vec<PathBuf>, depth: usize) {
        if depth > 4 {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if Self::is_framework_dir(&path) {
                    paths.push(path);
                } else {
                    Self::collect_core_paths(&path, paths, depth + 1);
                }
            } else if Self::is_libretro_library(&path) {
                paths.push(path);
            }
        }
    }

    fn is_framework_dir(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("framework"))
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
            display_name: fallback_name.clone(),
            ..CoreInfo::default()
        };

        if let Some(info_path) = self.find_info_path(core_path) {
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
                if let Some(val) = map
                    .get("manufacturer")
                    .or_else(|| map.get("system_manufacturer"))
                {
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
                if let Some(val) = map.get("database").or_else(|| map.get("databases")) {
                    info.database = val.clone();
                }
                info.firmware_count = map
                    .keys()
                    .filter(|key| key.starts_with("firmware") && key.ends_with("_path"))
                    .count();
            }
        }

        if info.supported_extensions.is_empty() || info.display_name == fallback_name {
            if let Some(probed) = Self::probe_system_info(core_path) {
                if info.display_name == fallback_name && !probed.display_name.is_empty() {
                    info.display_name = probed.display_name;
                }
                if info.display_version.is_empty() {
                    info.display_version = probed.display_version;
                }
                if info.supported_extensions.is_empty() {
                    info.supported_extensions = probed.supported_extensions;
                }
            }
        }
        info
    }

    fn probe_system_info(core_path: &Path) -> Option<CoreInfoProbe> {
        type RetroGetSystemInfo = unsafe extern "C" fn(*mut libretro::retro_system_info);
        let library = Library::open(core_path).ok()?;
        let symbol = CString::new("retro_get_system_info").ok()?;
        let get_system_info = library
            .symbol::<RetroGetSystemInfo>(symbol.as_c_str())
            .ok()?
            .get();
        let mut raw = libretro::retro_system_info {
            library_name: std::ptr::null(),
            library_version: std::ptr::null(),
            valid_extensions: std::ptr::null(),
            need_fullpath: false,
            block_extract: false,
        };
        unsafe { get_system_info(&mut raw) };
        Some(CoreInfoProbe {
            display_name: cstr_ptr_to_string(raw.library_name),
            display_version: cstr_ptr_to_string(raw.library_version),
            supported_extensions: cstr_ptr_to_string(raw.valid_extensions)
                .split('|')
                .map(|entry| entry.trim().trim_start_matches('.').to_lowercase())
                .filter(|entry| !entry.is_empty())
                .collect(),
        })
    }

    fn find_info_path(&self, core_path: &Path) -> Option<PathBuf> {
        if self.info_dir.as_os_str().is_empty() {
            return None;
        }

        for dir in self.info_search_dirs() {
            for candidate in Self::info_name_candidates(core_path) {
                let path = dir.join(format!("{candidate}.info"));
                if path.exists() {
                    return Some(path);
                }
            }
        }
        None
    }

    fn info_search_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if self.info_dir.as_os_str().is_empty() {
            return dirs;
        }
        Self::push_unique_path(&mut dirs, self.info_dir.clone());
        Self::push_unique_path(&mut dirs, self.info_dir.join("info"));
        Self::push_unique_path(&mut dirs, self.info_dir.join("assets/info"));
        if let Some(parent) = self.info_dir.parent() {
            Self::push_unique_path(&mut dirs, parent.join("assets/info"));
        }
        dirs
    }

    fn info_name_candidates(core_path: &Path) -> Vec<String> {
        let Some(stem) = core_path.file_stem().and_then(|stem| stem.to_str()) else {
            return Vec::new();
        };

        let mut bases = Vec::new();
        Self::push_unique(&mut bases, stem.to_string());

        if let Some(stripped) = stem.strip_prefix("lib") {
            Self::push_unique(&mut bases, stripped.to_string());
        }

        let separators_normalized = [stem.replace('-', "_"), stem.replace('.', "_")];
        for normalized in separators_normalized {
            if normalized != stem {
                Self::push_unique(&mut bases, normalized);
            }
        }

        let initial_bases = bases.clone();
        for base in initial_bases {
            Self::push_platform_normalized_bases(&mut bases, &base);
        }

        let mut candidates = Vec::new();
        for base in bases {
            Self::push_info_name_variants(&mut candidates, &base);
        }
        candidates
    }

    fn push_platform_normalized_bases(bases: &mut Vec<String>, name: &str) {
        for platform_suffix in ["_ios", "_macos", "_android"] {
            if let Some(base) = name.strip_suffix(platform_suffix) {
                Self::push_unique(bases, base.to_string());
                if let Some(core_name) = base.strip_suffix("_libretro") {
                    Self::push_unique(bases, core_name.to_string());
                    Self::push_unique(bases, format!("{core_name}_libretro"));
                }
                if let Some(core_name) = base.strip_suffix(".libretro") {
                    Self::push_unique(bases, core_name.to_string());
                    Self::push_unique(bases, format!("{core_name}_libretro"));
                }
            }
        }

        if let Some(core_name) = name.strip_suffix("_libretro") {
            Self::push_unique(bases, core_name.to_string());
        }

        if let Some(core_name) = name.strip_suffix(".libretro") {
            Self::push_unique(bases, core_name.to_string());
            Self::push_unique(bases, format!("{core_name}_libretro"));
        }
    }

    fn push_info_name_variants(candidates: &mut Vec<String>, name: &str) {
        let lower = name.to_lowercase();
        let underscore = lower.replace(['-', '.'], "_");
        let dotted = lower.replace(['-', '_'], ".");
        let hyphenated = lower.replace(['_', '.'], "-");

        for variant in [name.to_string(), lower, underscore, dotted, hyphenated] {
            Self::push_unique(candidates, variant.clone());
            if !variant.ends_with("_libretro")
                && !variant.ends_with(".libretro")
                && !variant.ends_with("-libretro")
            {
                Self::push_unique(candidates, format!("{variant}_libretro"));
            }
        }
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

    fn push_unique_path(values: &mut Vec<PathBuf>, value: PathBuf) {
        if !value.as_os_str().is_empty() && !values.iter().any(|existing| existing == &value) {
            values.push(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_ios_framework_bundles() {
        let dir = std::env::temp_dir().join(format!(
            "retrofront-framework-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let framework = dir.join("mgba_libretro_ios.framework");
        fs::create_dir_all(&framework).unwrap();
        File::create(framework.join("mgba_libretro_ios")).unwrap();

        let mut list = CoreInfoList::new();
        list.scan_directory(&dir);

        assert_eq!(list.cores.len(), 1);
        assert_eq!(list.cores[0].path, framework);
        assert!(list.cores[0].supported_extensions.is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn does_not_embed_core_extensions_without_info_file() {
        let list = CoreInfoList::new();
        let info = list.load_info_for_core(Path::new("/cores/mgba_libretro_ios.dylib"));
        assert!(info.supported_extensions.is_empty());
    }

    #[test]
    fn derives_retroarch_info_candidates_for_ios_libretro_names() {
        let candidates =
            CoreInfoList::info_name_candidates(Path::new("/cores/mgba_libretro_ios.dylib"));
        assert!(candidates.contains(&"mgba_libretro_ios".to_string()));
        assert!(candidates.contains(&"mgba_libretro".to_string()));
        assert!(candidates.contains(&"mgba".to_string()));
    }

    #[test]
    fn derives_retroarch_info_candidates_for_framework_names() {
        let candidates =
            CoreInfoList::info_name_candidates(Path::new("/cores/azahar.libretro.framework"));
        assert!(candidates.contains(&"azahar.libretro".to_string()));
        assert!(candidates.contains(&"azahar_libretro".to_string()));
        assert!(candidates.contains(&"azahar".to_string()));
    }

    #[test]
    fn derives_lowercase_libretro_info_candidates_for_plain_framework_names() {
        let candidates = CoreInfoList::info_name_candidates(Path::new("/cores/Citra.framework"));
        assert!(candidates.contains(&"Citra".to_string()));
        assert!(candidates.contains(&"citra".to_string()));
        assert!(candidates.contains(&"citra_libretro".to_string()));
    }

    #[test]
    fn finds_nested_assets_info_dir_from_configured_info_dir() {
        let dir = std::env::temp_dir().join(format!(
            "retrofront-nested-info-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(dir.join("assets/info")).unwrap();
        fs::write(
            dir.join("assets/info/azahar_libretro.info"),
            "display_name = \"Azahar\"\nsupported_extensions = \"3ds|3dsx|cci|cxi|app\"\n",
        )
        .unwrap();
        let mut list = CoreInfoList::new();
        list.set_info_dir(dir.join("info"));
        let info = list.load_info_for_core(Path::new("/cores/azahar.libretro.framework"));
        assert_eq!(info.display_name, "Azahar");
        assert!(info.supported_extensions.contains(&"3ds".to_string()));
        assert!(info.supported_extensions.contains(&"cci".to_string()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn finds_assets_zip_style_libretro_info_name() {
        let dir = std::env::temp_dir().join(format!(
            "retrofront-info-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(dir.join("info")).unwrap();
        fs::write(
            dir.join("info/mgba_libretro.info"),
            "display_name = \"mGBA\"\nsupported_extensions = \"gba|gb|gbc\"\n",
        )
        .unwrap();
        let mut list = CoreInfoList::new();
        list.set_info_dir(dir.clone());
        let info = list.load_info_for_core(Path::new("/cores/mgba_libretro_ios.dylib"));
        assert_eq!(info.display_name, "mGBA");
        assert!(info.supported_extensions.contains(&"gba".to_string()));
        let _ = fs::remove_dir_all(dir);
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

fn zip_member_extensions(path: &Path) -> Vec<String> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    let Ok(mut archive) = zip::ZipArchive::new(file) else {
        return Vec::new();
    };
    let mut extensions = Vec::new();
    for i in 0..archive.len() {
        let Ok(mut entry) = archive.by_index(i) else {
            continue;
        };
        if entry.is_dir() || entry.name().ends_with('/') {
            continue;
        }
        let mut probe = [0_u8; 1];
        let _ = entry.read(&mut probe);
        if let Some(ext) = Path::new(entry.name())
            .extension()
            .and_then(|ext| ext.to_str())
        {
            let ext = ext.trim_start_matches('.').to_lowercase();
            if !ext.is_empty() {
                extensions.push(ext);
            }
        }
    }
    extensions
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

#[derive(Debug, Clone, Default)]
struct CoreInfoProbe {
    display_name: String,
    display_version: String,
    supported_extensions: Vec<String>,
}

fn cstr_ptr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}
