use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Settings {
    pub values: HashMap<String, String>,
    pub path: PathBuf,
    pub base_dir: PathBuf,
}

impl Settings {
    pub fn new() -> Self {
        let mut settings = Self::default();
        settings.apply_retroarch_defaults(Path::new("."));
        settings
    }

    pub fn load(&mut self, path: &Path) {
        self.path = path.to_path_buf();
        if let Some(parent) = path.parent() {
            self.base_dir = parent.to_path_buf();
            self.apply_retroarch_defaults(parent);
        }
        if !path.exists() {
            return;
        }

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    self.values
                        .insert(key.trim().to_string(), Self::unquote(value.trim()));
                }
            }
        }
    }

    pub fn save(&self) {
        if self.path.as_os_str().is_empty() {
            return;
        }
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
        {
            let mut keys: Vec<&String> = self.values.keys().collect();
            keys.sort();
            for key in keys {
                let value = self.values.get(key).unwrap();
                let escaped = value.replace('"', "\\\"");
                let _ = writeln!(file, "{} = \"{}\"", key, escaped);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn set_base_dir(&mut self, base_dir: &Path) {
        self.base_dir = base_dir.to_path_buf();
        for key in Self::managed_default_keys() {
            self.values.remove(*key);
        }
        self.apply_retroarch_defaults(base_dir);
    }

    pub fn ensure_directories(&self) {
        for key in Self::directory_keys() {
            if let Some(path) = self.path_value(key) {
                let _ = fs::create_dir_all(path);
            }
        }
    }

    pub fn path_value(&self, key: &str) -> Option<PathBuf> {
        self.values.get(key).map(PathBuf::from)
    }

    pub fn libretro_info_path(&self) -> PathBuf {
        self.path_value("libretro_info_path")
            .unwrap_or_else(|| self.base_dir.join("info"))
    }

    pub fn libretro_directory(&self) -> PathBuf {
        self.path_value("libretro_directory")
            .unwrap_or_else(|| self.base_dir.join("cores"))
    }

    pub fn content_directory(&self) -> PathBuf {
        self.path_value("content_directory")
            .unwrap_or_else(|| self.base_dir.join("roms"))
    }

    pub fn savefile_directory(&self) -> PathBuf {
        self.path_value("savefile_directory")
            .unwrap_or_else(|| self.base_dir.join("saves"))
    }

    pub fn savestate_directory(&self) -> PathBuf {
        self.path_value("savestate_directory")
            .unwrap_or_else(|| self.base_dir.join("states"))
    }

    pub fn system_directory(&self) -> PathBuf {
        self.path_value("system_directory")
            .unwrap_or_else(|| self.base_dir.join("system"))
    }

    pub fn preferred_core_for_extension(&self, extension: &str) -> Option<PathBuf> {
        let ext = extension.trim_start_matches('.').to_lowercase();
        if ext.is_empty() {
            return None;
        }
        self.path_value(&format!("content_core_{}", ext))
    }

    pub fn set_preferred_core_for_extension(&mut self, extension: &str, core_path: &Path) {
        let ext = extension.trim_start_matches('.').to_lowercase();
        if !ext.is_empty() {
            self.set(
                &format!("content_core_{}", ext),
                &core_path.to_string_lossy(),
            );
        }
    }

    pub fn runtime_directory(&self) -> PathBuf {
        self.path_value("runtime_directory")
            .unwrap_or_else(|| self.base_dir.join("runtime"))
    }

    pub fn cache_directory(&self) -> PathBuf {
        self.path_value("cache_directory")
            .unwrap_or_else(|| self.base_dir.join("cache"))
    }

    pub fn thumbnails_directory(&self) -> PathBuf {
        self.path_value("thumbnails_directory")
            .unwrap_or_else(|| self.base_dir.join("thumbnails"))
    }

    pub fn menu_assets_directory(&self) -> PathBuf {
        self.path_value("menu_assets_directory")
            .or_else(|| self.path_value("assets_directory"))
            .unwrap_or_else(|| self.base_dir.join("assets"))
    }

    fn apply_retroarch_defaults(&mut self, base_dir: &Path) {
        let defaults = [
            ("libretro_directory", base_dir.join("Cores")),
            ("libretro_info_path", base_dir.join("info")),
            (
                "core_options_path",
                base_dir.join("retroarch-core-options.cfg"),
            ),
            ("content_directory", base_dir.join("Roms")),
            ("savefile_directory", base_dir.join("saves")),
            ("savestate_directory", base_dir.join("states")),
            ("system_directory", base_dir.join("system")),
            ("playlist_directory", base_dir.join("playlists")),
            ("core_assets_directory", base_dir.join("downloads")),
            ("assets_directory", base_dir.join("assets")),
            ("menu_assets_directory", base_dir.join("assets")),
            ("thumbnails_directory", base_dir.join("thumbnails")),
            ("runtime_directory", base_dir.join("runtime")),
            ("cache_directory", base_dir.join("cache")),
            ("screenshot_directory", base_dir.join("screenshots")),
            ("input_remapping_directory", base_dir.join("remaps")),
            ("cheat_database_path", base_dir.join("cheats")),
            ("overlay_directory", base_dir.join("overlays")),
            ("joypad_autoconfig_dir", base_dir.join("autoconfig")),
            ("video_filter_dir", base_dir.join("filters/video")),
            ("audio_filter_dir", base_dir.join("filters/audio")),
            ("log_dir", base_dir.join("logs")),
            ("video_driver", PathBuf::from("bgfx")),
            ("audio_driver", PathBuf::from("swift")),
            ("input_driver", PathBuf::from("swift")),
            ("menu_driver", PathBuf::from("xmb")),
            ("menu_xmb_theme", PathBuf::from("monochrome")),
        ];
        for (key, value) in defaults {
            let value = value.to_string_lossy().into_owned();
            match self.values.get(key) {
                Some(existing) if !existing.starts_with("./") => {}
                _ => {
                    self.values.insert(key.to_string(), value);
                }
            }
        }
    }

    fn managed_default_keys() -> &'static [&'static str] {
        &[
            "core_options_path",
            "video_driver",
            "audio_driver",
            "input_driver",
            "menu_driver",
            "menu_xmb_theme",
            "libretro_directory",
            "libretro_info_path",
            "content_directory",
            "savefile_directory",
            "savestate_directory",
            "system_directory",
            "playlist_directory",
            "core_assets_directory",
            "assets_directory",
            "menu_assets_directory",
            "thumbnails_directory",
            "runtime_directory",
            "cache_directory",
            "screenshot_directory",
            "input_remapping_directory",
            "cheat_database_path",
            "overlay_directory",
            "joypad_autoconfig_dir",
            "video_filter_dir",
            "audio_filter_dir",
            "log_dir",
        ]
    }

    fn directory_keys() -> &'static [&'static str] {
        &Self::managed_default_keys()[6..]
    }

    fn unquote(value: &str) -> String {
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .unwrap_or(value)
            .replace("\\\"", "\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_base_dir_rebases_managed_defaults() {
        let mut settings = Settings::new();
        settings.set_base_dir(Path::new("/tmp/RetrofrontA"));
        settings.set_base_dir(Path::new("/tmp/RetrofrontB"));
        assert_eq!(
            settings.libretro_directory(),
            PathBuf::from("/tmp/RetrofrontB/Cores")
        );
    }

    #[test]
    fn retroarch_directory_defaults_are_rooted_in_base_dir() {
        let mut settings = Settings::new();
        settings.set_base_dir(Path::new("/tmp/Retrofront"));
        assert_eq!(
            settings.libretro_directory(),
            PathBuf::from("/tmp/Retrofront/Cores")
        );
        assert_eq!(
            settings.system_directory(),
            PathBuf::from("/tmp/Retrofront/system")
        );
    }
}
