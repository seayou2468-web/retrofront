use crate::core_info::CoreInfo;
use crate::settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntryKind {
    Action,
    Submenu,
    Toggle,
    Setting,
}

#[derive(Debug, Clone)]
pub struct MenuEntry {
    pub label: String,
    pub sublabel: String,
    pub kind: MenuEntryKind,
    pub value: String,
    pub action_id: u32,
}

#[derive(Debug, Clone)]
pub struct MenuList {
    pub title: String,
    pub entries: Vec<MenuEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSkin {
    pub driver: String,
    pub theme: String,
    pub assets_directory: String,
}

pub struct MenuEngine {
    pub history: Vec<MenuList>,
    pub skin: MenuSkin,
}

impl MenuEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            history: Vec::new(),
            skin: MenuSkin {
                driver: "xmb".to_string(),
                theme: "monochrome".to_string(),
                assets_directory: String::new(),
            },
        };
        engine.push_main_menu();
        engine
    }

    pub fn push_main_menu(&mut self) {
        self.history.clear();
        self.history.push(MenuList {
            title: "Retrofront".to_string(),
            entries: vec![
                Self::submenu("Load Core", "Select a libretro core", 1),
                Self::submenu("Load Content", "Browse scanned content", 2),
                Self::submenu("Online Updater", "Refresh core info and assets", 3),
                Self::submenu(
                    "Settings",
                    "Video, audio, input, directories and core settings",
                    4,
                ),
                Self::submenu("Information", "Core, content and system metadata", 5),
            ],
        });
    }

    pub fn push_quick_menu(&mut self, has_game: bool) {
        let mut entries = vec![
            Self::action("Resume", "Continue playing", 10),
            Self::submenu(
                "Core Options",
                "Adjust variables exposed by the active core",
                11,
            ),
            Self::submenu("Shaders", "Configure video filters and scaling", 13),
            Self::submenu("Save States", "Save, load and manage state slots", 14),
        ];
        if has_game {
            entries.push(Self::action("Close Content", "Unload the current game", 12));
        }
        self.history.push(MenuList {
            title: "Quick Menu".to_string(),
            entries,
        });
    }

    pub fn push_core_list(&mut self, cores: &[CoreInfo]) {
        let entries = cores
            .iter()
            .enumerate()
            .map(|(i, core)| MenuEntry {
                label: core.display_name.clone(),
                sublabel: if core.system_name.is_empty() {
                    core.path.to_string_lossy().into_owned()
                } else {
                    format!(
                        "{} • {}",
                        core.system_name,
                        core.supported_extensions.join(", ")
                    )
                },
                kind: MenuEntryKind::Action,
                value: core.path.to_string_lossy().into_owned(),
                action_id: 100 + i as u32,
            })
            .collect();
        self.history.push(MenuList {
            title: "Load Core".to_string(),
            entries,
        });
    }

    pub fn apply_skin_from_settings(&mut self, settings: &Settings) {
        self.skin = MenuSkin {
            driver: settings
                .get("menu_driver")
                .cloned()
                .unwrap_or_else(|| "xmb".to_string()),
            theme: settings
                .get("menu_xmb_theme")
                .cloned()
                .unwrap_or_else(|| "monochrome".to_string()),
            assets_directory: settings
                .menu_assets_directory()
                .to_string_lossy()
                .into_owned(),
        };
    }

    pub fn push_skin_settings(&mut self, settings: &Settings) {
        self.apply_skin_from_settings(settings);
        let entries = vec![
            Self::setting(
                "Menu Driver",
                "RetroArch-compatible menu driver id",
                &self.skin.driver,
                260,
            ),
            Self::setting(
                "XMB Theme",
                "Icon and background theme",
                &self.skin.theme,
                261,
            ),
            Self::setting(
                "Menu Assets",
                "XMB/Ozone asset root",
                &self.skin.assets_directory,
                262,
            ),
        ];
        self.history.push(MenuList {
            title: "XMB Theme".to_string(),
            entries,
        });
    }

    pub fn push_settings(&mut self, settings: &Settings) {
        let entries = vec![
            Self::setting(
                "Core Directory",
                "Where libretro dylibs are discovered",
                settings.libretro_directory().to_string_lossy(),
                200,
            ),
            Self::setting(
                "Core Info Directory",
                "RetroArch .info metadata path",
                settings.libretro_info_path().to_string_lossy(),
                201,
            ),
            Self::setting(
                "Content Directory",
                "ROM/content browser root",
                settings.content_directory().to_string_lossy(),
                202,
            ),
            Self::setting(
                "System/BIOS Directory",
                "Firmware and BIOS files",
                settings.system_directory().to_string_lossy(),
                203,
            ),
            Self::setting(
                "Savefile Directory",
                "SRAM and memory card saves",
                settings.savefile_directory().to_string_lossy(),
                204,
            ),
            Self::setting(
                "Savestate Directory",
                "Instant save states",
                settings.savestate_directory().to_string_lossy(),
                205,
            ),
            Self::setting(
                "Playlist Directory",
                "Scanned content playlists",
                settings
                    .path_value("playlist_directory")
                    .unwrap_or_default()
                    .to_string_lossy(),
                206,
            ),
            Self::setting(
                "Cache Directory",
                "Temporary extraction and runtime files",
                settings.cache_directory().to_string_lossy(),
                207,
            ),
            Self::setting(
                "Thumbnails Directory",
                "Box art and media thumbnails",
                settings.thumbnails_directory().to_string_lossy(),
                208,
            ),
        ];
        self.history.push(MenuList {
            title: "Settings".to_string(),
            entries,
        });
    }

    pub fn pop(&mut self) -> Option<MenuList> {
        if self.history.len() > 1 {
            self.history.pop()
        } else {
            None
        }
    }

    pub fn current(&self) -> Option<&MenuList> {
        self.history.last()
    }

    pub fn clear_to_main(&mut self) {
        self.history.truncate(1);
    }

    fn action(label: &str, sublabel: &str, action_id: u32) -> MenuEntry {
        MenuEntry {
            label: label.to_string(),
            sublabel: sublabel.to_string(),
            kind: MenuEntryKind::Action,
            value: String::new(),
            action_id,
        }
    }

    fn submenu(label: &str, sublabel: &str, action_id: u32) -> MenuEntry {
        MenuEntry {
            label: label.to_string(),
            sublabel: sublabel.to_string(),
            kind: MenuEntryKind::Submenu,
            value: String::new(),
            action_id,
        }
    }

    fn setting(
        label: &str,
        sublabel: &str,
        value: impl std::fmt::Display,
        action_id: u32,
    ) -> MenuEntry {
        MenuEntry {
            label: label.to_string(),
            sublabel: sublabel.to_string(),
            kind: MenuEntryKind::Setting,
            value: value.to_string(),
            action_id,
        }
    }
}
