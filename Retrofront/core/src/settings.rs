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
            let base_dir = if parent.file_name().and_then(|name| name.to_str()) == Some("config") {
                parent.parent().unwrap_or(parent)
            } else {
                parent
            };
            self.base_dir = base_dir.to_path_buf();
            self.apply_retroarch_defaults(base_dir);
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

    pub fn string_value(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    pub fn bool_value(&self, key: &str) -> Option<bool> {
        self.values.get(key).map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "true" | "1" | "yes" | "on"
            )
        })
    }

    pub fn float_value(&self, key: &str) -> Option<f32> {
        self.values.get(key).and_then(|value| value.parse().ok())
    }

    pub fn libretro_info_path(&self) -> PathBuf {
        self.path_value("libretro_info_path")
            .unwrap_or_else(|| self.menu_assets_directory().join("info"))
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
            ("libretro_info_path", base_dir.join("assets/info")),
            (
                "core_options_path",
                base_dir.join("retroarch-core-options.cfg"),
            ),
            ("content_directory", base_dir.join("Roms")),
            ("savefile_directory", base_dir.join("saves")),
            ("savestate_directory", base_dir.join("states")),
            ("system_directory", base_dir.join("system")),
            ("playlist_directory", base_dir.join("playlists")),
            ("menu_content_directory", base_dir.to_path_buf()),
            (
                "content_favorites_path",
                base_dir.join("playlists/content_favorites.lpl"),
            ),
            (
                "content_history_path",
                base_dir.join("playlists/content_history.lpl"),
            ),
            (
                "content_image_history_path",
                base_dir.join("playlists/content_image_history.lpl"),
            ),
            (
                "content_music_history_path",
                base_dir.join("playlists/content_music_history.lpl"),
            ),
            (
                "content_video_history_path",
                base_dir.join("playlists/content_video_history.lpl"),
            ),
            ("core_assets_directory", base_dir.join("downloads")),
            ("assets_directory", base_dir.join("assets")),
            ("menu_assets_directory", base_dir.join("assets")),
            ("thumbnails_directory", base_dir.join("thumbnails")),
            ("runtime_directory", base_dir.join("runtime")),
            ("cache_directory", base_dir.join("cache")),
            ("screenshot_directory", base_dir.join("screenshots")),
            ("input_remapping_directory", base_dir.join("remaps")),
            ("cheat_database_path", base_dir.join("cht")),
            ("content_database_path", base_dir.join("database/rdb")),
            ("overlay_directory", base_dir.join("assets/overlays")),
            (
                "input_overlay",
                base_dir.join("assets/overlays/gamepads/flat/retropad.cfg"),
            ),
            ("joypad_autoconfig_dir", base_dir.join("autoconfig")),
            ("video_shader_dir", base_dir.join("shaders")),
            ("video_filter_dir", base_dir.join("filters/video")),
            ("audio_filter_dir", base_dir.join("filters/audio")),
            ("log_dir", base_dir.join("logs")),
            ("recording_output_directory", base_dir.join("records")),
            (
                "recording_config_directory",
                base_dir.join("records_config"),
            ),
            ("dynamic_wallpapers_directory", base_dir.join("wallpapers")),
            ("video_driver", PathBuf::from("metal")),
            ("video_bgfx_renderer", PathBuf::from("metal")),
            ("audio_driver", PathBuf::from("swift")),
            ("input_driver", PathBuf::from("swift")),
            ("menu_driver", PathBuf::from("oneui")),
            ("menu_theme", PathBuf::from("dark")),
            ("menu_color_scheme", PathBuf::from("dark")),
            ("menu_layout_density", PathBuf::from("standard")),
            ("menu_card_style", PathBuf::from("modern")),
            ("play_screen_orientation", PathBuf::from("auto")),
            ("play_screen_portrait_layout", PathBuf::from("fit")),
            ("play_screen_landscape_layout", PathBuf::from("immersive")),
            ("quick_menu_style", PathBuf::from("oneui_fullscreen")),
            ("library_mode", PathBuf::from("roms_only")),
            ("library_sort_mode", PathBuf::from("name_ascending")),
            ("library_show_core_badges", PathBuf::from("true")),
            ("library_show_file_details", PathBuf::from("true")),
            ("library_auto_scan_on_launch", PathBuf::from("true")),
            ("video_scale_mode", PathBuf::from("keep_aspect")),
            ("video_filter_mode", PathBuf::from("nearest")),
            ("video_vsync", PathBuf::from("true")),
            ("audio_enable", PathBuf::from("true")),
            ("audio_sync", PathBuf::from("true")),
            ("audio_latency_ms", PathBuf::from("64")),
            ("input_haptic_feedback", PathBuf::from("true")),
            ("savestate_auto_save", PathBuf::from("false")),
            ("savestate_auto_load", PathBuf::from("false")),
            ("rewind_enable", PathBuf::from("false")),
            ("fastforward_ratio", PathBuf::from("0.0")),
        ];
        let scalar_defaults = [
            ("input_overlay_enable", "true"),
            ("input_overlay_opacity", "0.70"),
            ("input_overlay_scale", "1.0"),
        ];
        for (key, value) in scalar_defaults {
            self.values
                .entry(key.to_string())
                .or_insert_with(|| value.to_string());
        }
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
            "input_overlay_enable",
            "input_overlay_opacity",
            "input_overlay_scale",
            "menu_driver",
            "menu_theme",
            "menu_color_scheme",
            "menu_layout_density",
            "menu_card_style",
            "play_screen_orientation",
            "play_screen_portrait_layout",
            "play_screen_landscape_layout",
            "quick_menu_style",
            "library_mode",
            "library_sort_mode",
            "library_show_core_badges",
            "library_show_file_details",
            "library_auto_scan_on_launch",
            "video_scale_mode",
            "video_filter_mode",
            "video_vsync",
            "audio_enable",
            "audio_sync",
            "audio_latency_ms",
            "input_haptic_feedback",
            "savestate_auto_save",
            "savestate_auto_load",
            "rewind_enable",
            "fastforward_ratio",
            "libretro_directory",
            "libretro_info_path",
            "content_directory",
            "savefile_directory",
            "savestate_directory",
            "system_directory",
            "playlist_directory",
            "menu_content_directory",
            "content_favorites_path",
            "content_history_path",
            "content_image_history_path",
            "content_music_history_path",
            "content_video_history_path",
            "core_assets_directory",
            "assets_directory",
            "menu_assets_directory",
            "thumbnails_directory",
            "runtime_directory",
            "cache_directory",
            "screenshot_directory",
            "input_remapping_directory",
            "cheat_database_path",
            "content_database_path",
            "overlay_directory",
            "input_overlay",
            "joypad_autoconfig_dir",
            "video_shader_dir",
            "video_filter_dir",
            "audio_filter_dir",
            "log_dir",
            "recording_output_directory",
            "recording_config_directory",
            "dynamic_wallpapers_directory",
        ]
    }

    fn directory_keys() -> &'static [&'static str] {
        &[
            "libretro_directory",
            "libretro_info_path",
            "content_directory",
            "savefile_directory",
            "savestate_directory",
            "system_directory",
            "playlist_directory",
            "menu_content_directory",
            "core_assets_directory",
            "assets_directory",
            "menu_assets_directory",
            "thumbnails_directory",
            "runtime_directory",
            "cache_directory",
            "screenshot_directory",
            "input_remapping_directory",
            "cheat_database_path",
            "content_database_path",
            "overlay_directory",
            "input_overlay",
            "joypad_autoconfig_dir",
            "video_shader_dir",
            "video_filter_dir",
            "audio_filter_dir",
            "log_dir",
            "recording_output_directory",
            "recording_config_directory",
            "dynamic_wallpapers_directory",
        ]
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
        assert_eq!(
            settings.libretro_info_path(),
            PathBuf::from("/tmp/Retrofront/assets/info")
        );
    }
}
