use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathConfig {
    pub system_dir: PathBuf,
    pub save_dir: PathBuf,
    pub state_dir: PathBuf,
    pub skin_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendConfig {
    pub paths: PathConfig,
    pub core_options: BTreeMap<String, String>,
    pub menu_driver: String,
    pub video_driver: String,
    pub audio_driver: String,
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            paths: PathConfig {
                system_dir: PathBuf::from("system"),
                save_dir: PathBuf::from("saves"),
                state_dir: PathBuf::from("states"),
                skin_dir: PathBuf::from("assets/skins"),
            },
            core_options: BTreeMap::new(),
            menu_driver: "xmb".into(),
            video_driver: "swift-metal-or-vulkan".into(),
            audio_driver: "swift-platform-audio".into(),
        }
    }
}

impl FrontendConfig {
    pub fn load_retroarch_cfg(path: &Path) -> Result<Self, String> {
        let mut cfg = Self::default();
        let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!(
                    "{}:{}: expected key = value",
                    path.display(),
                    line_no + 1
                ));
            };
            let key = key.trim();
            let value = value.trim().trim_matches('"').to_string();
            match key {
                "system_directory" => cfg.paths.system_dir = value.into(),
                "savefile_directory" => cfg.paths.save_dir = value.into(),
                "savestate_directory" => cfg.paths.state_dir = value.into(),
                "assets_directory" => cfg.paths.skin_dir = value.into(),
                "menu_driver" => cfg.menu_driver = value,
                "video_driver" => cfg.video_driver = value,
                "audio_driver" => cfg.audio_driver = value,
                k if k.starts_with("core_option_") => {
                    cfg.core_options
                        .insert(k.trim_start_matches("core_option_").into(), value);
                }
                _ => {}
            }
        }
        Ok(cfg)
    }

    pub fn ensure_directories(&self) -> Result<(), String> {
        for path in [
            &self.paths.system_dir,
            &self.paths.save_dir,
            &self.paths.state_dir,
        ] {
            fs::create_dir_all(path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_retroarch_style_config() {
        let dir = std::env::temp_dir();
        let file = dir.join(format!("retrofront-{}.cfg", std::process::id()));
        fs::write(&file, "menu_driver = \"rgui\"\nsystem_directory = \"sys\"\ncore_option_gambatte_gb_colorization = \"disabled\"\n").unwrap();
        let cfg = FrontendConfig::load_retroarch_cfg(&file).unwrap();
        assert_eq!(cfg.menu_driver, "rgui");
        assert_eq!(cfg.paths.system_dir, PathBuf::from("sys"));
        assert_eq!(cfg.core_options["gambatte_gb_colorization"], "disabled");
        let _ = fs::remove_file(file);
    }
}
