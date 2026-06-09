use std::path::PathBuf;
use crate::core_info::CoreInfo;
use crate::gfx::{GfxBackendKind, GfxStatus};
use crate::scanner::GameEntry;
use crate::settings::Settings;
use crate::{GameInfo, SystemInfo};

pub const ACTION_LOAD_CORE: u32 = 1;
pub const ACTION_LOAD_CONTENT: u32 = 2;
pub const ACTION_ONLINE_UPDATER: u32 = 3;
pub const ACTION_SETTINGS: u32 = 4;
pub const ACTION_INFORMATION: u32 = 5;
pub const ACTION_CONFIGURATION_FILE: u32 = 6;
pub const ACTION_HELP: u32 = 7;
pub const ACTION_QUICK_MENU: u32 = 8;
pub const ACTION_RESTART_CONTENT: u32 = 9;
pub const ACTION_RESUME_CONTENT: u32 = 10;
pub const ACTION_CORE_OPTIONS: u32 = 11;
pub const ACTION_CLOSE_CONTENT: u32 = 12;
pub const ACTION_SHADERS: u32 = 13;
pub const ACTION_SAVE_STATES: u32 = 14;
pub const ACTION_TAKE_SCREENSHOT: u32 = 15;
pub const ACTION_ADD_TO_FAVORITES: u32 = 16;
pub const ACTION_CHEATS: u32 = 17;
pub const ACTION_OVERRIDES: u32 = 18;
pub const ACTION_CONTROLS: u32 = 19;
pub const ACTION_CORE_INFORMATION: u32 = 20;
pub const ACTION_EXTRACT_ASSETS: u32 = 21;
pub const ACTION_CORE_SETTINGS: u32 = 22;

pub const ACTION_SETTINGS_DRIVERS: u32 = 210;
pub const ACTION_SETTINGS_VIDEO: u32 = 211;
pub const ACTION_SETTINGS_AUDIO: u32 = 212;
pub const ACTION_SETTINGS_INPUT: u32 = 213;
pub const ACTION_SETTINGS_DIRECTORIES: u32 = 214;
pub const ACTION_SETTINGS_USER_INTERFACE: u32 = 215;
pub const ACTION_SETTINGS_SAVING: u32 = 216;
pub const ACTION_SETTINGS_LATENCY: u32 = 217;
pub const ACTION_SETTINGS_FRAME_THROTTLE: u32 = 218;
pub const ACTION_SETTINGS_PLAYLISTS: u32 = 219;
pub const ACTION_SKIN_SETTINGS: u32 = 260;

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
                driver: "OneUI".to_string(),
                theme: "Dark".to_string(),
                assets_directory: String::new(),
            },
        };
        engine.push_main_menu();
        engine
    }

    pub fn push_main_menu(&mut self) {
        self.history.clear();
        self.history.push(MenuList {
            title: "Main Menu".to_string(),
            entries: vec![
                Self::submenu(
                    "Load Content",
                    "Browse scanned content",
                    ACTION_LOAD_CONTENT,
                ),
                Self::submenu(
                    "Quick Menu",
                    "Runtime actions for loaded content",
                    ACTION_QUICK_MENU,
                ),
                Self::submenu(
                    "Settings",
                    "Drivers, video, audio, input and directories",
                    ACTION_SETTINGS,
                ),
                Self::submenu(
                    "Information",
                    "Core, content, frontend and system metadata",
                    ACTION_INFORMATION,
                ),
            ],
        });
    }

    pub fn push_quick_menu(&mut self, has_game: bool) {
        let mut entries = vec![
            Self::action("Resume", "Continue playing", ACTION_RESUME_CONTENT),
            Self::submenu("Core Settings", "Manage core-specific options", ACTION_CORE_SETTINGS),
            Self::action(
                "Restart",
                "Reset the current content",
                ACTION_RESTART_CONTENT,
            ),
            Self::action("Close Content", "Unload core and content", ACTION_CLOSE_CONTENT),
        ];
        if !has_game {
            entries.clear();
            entries.push(Self::action("No Content Loaded", "", 0));
        }
        self.history.push(MenuList {
            title: "Quick Menu".to_string(),
            entries,
        });
    }

    pub fn push_settings(&mut self, _settings: &Settings) {
        self.history.push(MenuList {
            title: "Settings".to_string(),
            entries: vec![
                Self::action("Extract Assets", "Unpack bundled assets.zip", ACTION_EXTRACT_ASSETS),
                Self::submenu("Drivers", "Change hardware drivers", ACTION_SETTINGS_DRIVERS),
                Self::submenu("Video", "Video output and scaling", ACTION_SETTINGS_VIDEO),
                Self::submenu("Audio", "Audio output and synchronization", ACTION_SETTINGS_AUDIO),
                Self::submenu("Input", "Input, joypads and overlays", ACTION_SETTINGS_INPUT),
                Self::submenu("User Interface", "Menu and OSD settings", ACTION_SETTINGS_USER_INTERFACE),
                Self::submenu("Directories", "Paths for content, saves and assets", ACTION_SETTINGS_DIRECTORIES),
                Self::submenu("Skin Settings", "Appearance and themes", ACTION_SKIN_SETTINGS),
            ],
        });
    }

    pub fn push_status(&mut self, title: &str, message: &str) {
        self.history.push(MenuList {
            title: title.to_string(),
            entries: vec![Self::action(message, "", 0)],
        });
    }

    pub fn push_core_list(&mut self, cores: &[CoreInfo]) {
        let entries = cores
            .iter()
            .map(|core| Self::action(&core.display_name, &core.system_name, ACTION_LOAD_CORE))
            .collect();
        self.history.push(MenuList {
            title: "Load Core".to_string(),
            entries,
        });
    }

    pub fn push_content_list(&mut self, games: &[GameEntry]) {
        let entries = games
            .iter()
            .map(|game| Self::action(&game.label, &game.path.to_string_lossy(), ACTION_LOAD_CONTENT))
            .collect();
        self.history.push(MenuList {
            title: "Load Content".to_string(),
            entries,
        });
    }

    pub fn push_information(
        &mut self,
        system_info: Option<&SystemInfo>,
        game_info: Option<&GameInfo>,
        _gfx_status: &GfxStatus,
    ) {
        let content = game_info
            .map(|info| PathBuf::from(&info.path))
            .unwrap_or_else(|| PathBuf::from("Not loaded"));

        let entries = vec![
            Self::setting(
                "Core",
                "Loaded libretro core",
                system_info.map_or("Not loaded", |info| info.library_name.as_str()),
                400,
            ),
            Self::setting("Content", "Loaded content path", content.to_string_lossy(), 402),
        ];
        self.history.push(MenuList {
            title: "Information".to_string(),
            entries,
        });
    }

    pub fn push_skin_settings(&mut self, _settings: &Settings) {
        self.history.push(MenuList {
            title: "Skin Settings".to_string(),
            entries: vec![
                Self::setting("Theme", "Active UI theme", "Dark", 261),
                Self::setting("Icon Set", "Menu icon style", "Modern", 262),
            ],
        });
    }

    pub fn push_driver_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Drivers".to_string(),
            entries: vec![
                Self::setting(
                    "Video",
                    "Active video driver",
                    settings.get("video_driver").map_or("bgfx", String::as_str),
                    600,
                ),
                Self::setting(
                    "Audio",
                    "Active audio driver",
                    settings.get("audio_driver").map_or("swift", String::as_str),
                    601,
                ),
                Self::setting(
                    "Input",
                    "Active input driver",
                    settings.get("input_driver").map_or("swift", String::as_str),
                    602,
                ),
            ],
        });
    }

    pub fn push_video_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Video".to_string(),
            entries: vec![
                Self::setting(
                    "Output",
                    "Active rendering backend",
                    settings.get("video_driver").map_or("bgfx", String::as_str),
                    620,
                ),
            ],
        });
    }

    pub fn push_audio_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Audio".to_string(),
            entries: vec![
                Self::setting(
                    "Output",
                    "Active audio driver",
                    settings.get("audio_driver").map_or("swift", String::as_str),
                    640,
                ),
            ],
        });
    }

    pub fn push_input_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Input".to_string(),
            entries: vec![
                Self::setting(
                    "Driver",
                    "Active input driver",
                    settings.get("input_driver").map_or("swift", String::as_str),
                    660,
                ),
            ],
        });
    }

    pub fn push_directory_settings(&mut self, settings: &Settings) {
        let entries = vec![
            Self::setting(
                "Core Directory",
                "Where libretro dylibs are discovered",
                settings.libretro_directory().to_string_lossy(),
                200,
            ),
            Self::setting(
                "Content Directory",
                "ROM/content browser root",
                settings.content_directory().to_string_lossy(),
                202,
            ),
        ];
        self.history.push(MenuList {
            title: "Directories".to_string(),
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
