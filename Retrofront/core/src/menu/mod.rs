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
            title: "Main Menu".to_string(),
            entries: vec![
                Self::submenu("Load Core", "Select a libretro core", ACTION_LOAD_CORE),
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
                    "Online Updater",
                    "Core info, assets and databases",
                    ACTION_ONLINE_UPDATER,
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
                Self::submenu(
                    "Configuration File",
                    "RetroArch-style configuration actions",
                    ACTION_CONFIGURATION_FILE,
                ),
                Self::submenu(
                    "Help",
                    "Basic menu usage and RetroPad controls",
                    ACTION_HELP,
                ),
            ],
        });
    }

    pub fn push_quick_menu(&mut self, has_game: bool) {
        let mut entries = vec![
            Self::action("Resume", "Continue playing", ACTION_RESUME_CONTENT),
            Self::action(
                "Restart",
                "Reset the current content",
                ACTION_RESTART_CONTENT,
            ),
            Self::submenu(
                "Core Options",
                "Adjust variables exposed by the active core",
                ACTION_CORE_OPTIONS,
            ),
            Self::submenu(
                "Controls",
                "Per-core and per-content input remaps",
                ACTION_CONTROLS,
            ),
            Self::submenu(
                "Shaders",
                "Configure video shader passes and presets",
                ACTION_SHADERS,
            ),
            Self::submenu(
                "Save States",
                "Save, load and manage state slots",
                ACTION_SAVE_STATES,
            ),
            Self::submenu("Cheats", "Load and apply cheat files", ACTION_CHEATS),
            Self::submenu(
                "Overrides",
                "Core/content/game override configuration",
                ACTION_OVERRIDES,
            ),
            Self::action(
                "Take Screenshot",
                "Write a screenshot to the screenshots directory",
                ACTION_TAKE_SCREENSHOT,
            ),
            Self::action(
                "Add to Favorites",
                "Add current content to Favorites playlist",
                ACTION_ADD_TO_FAVORITES,
            ),
            Self::submenu(
                "Information",
                "Core and content runtime details",
                ACTION_CORE_INFORMATION,
            ),
        ];
        if has_game {
            entries.push(Self::action(
                "Close Content",
                "Unload the current game",
                ACTION_CLOSE_CONTENT,
            ));
        }
        self.history.push(MenuList {
            title: "Quick Menu".to_string(),
            entries,
        });
    }

    pub fn push_core_list(&mut self, cores: &[CoreInfo]) {
        let entries = if cores.is_empty() {
            vec![Self::action(
                "No Cores Available",
                "Scan the configured core directory or import a .dylib core",
                0,
            )]
        } else {
            cores
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
                .collect()
        };
        self.history.push(MenuList {
            title: "Load Core".to_string(),
            entries,
        });
    }

    pub fn push_content_list(&mut self, games: &[GameEntry]) {
        let entries = if games.is_empty() {
            vec![Self::action(
                "No Content Found",
                "Import or scan ROMs from the configured content directory",
                0,
            )]
        } else {
            games
                .iter()
                .enumerate()
                .map(|(i, game)| MenuEntry {
                    label: game.label.clone(),
                    sublabel: game.path.to_string_lossy().into_owned(),
                    kind: MenuEntryKind::Action,
                    value: game.path.to_string_lossy().into_owned(),
                    action_id: 300 + i as u32,
                })
                .collect()
        };
        self.history.push(MenuList {
            title: "Load Content".to_string(),
            entries,
        });
    }

    pub fn push_information(
        &mut self,
        system_info: Option<&SystemInfo>,
        game_info: Option<&GameInfo>,
        gfx_status: &GfxStatus,
    ) {
        let backend = gfx_status
            .last_present
            .as_ref()
            .map(|status| match status.backend {
                GfxBackendKind::Software => "software",
                GfxBackendKind::Bgfx => "bgfx",
            })
            .unwrap_or("not rendered yet");
        let content = game_info
            .map(|info| info.path.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Not loaded".to_string());
        let mut entries = vec![
            Self::setting(
                "Core",
                "Loaded libretro core",
                system_info.map_or("Not loaded", |info| info.library_name.as_str()),
                400,
            ),
            Self::setting(
                "Core Version",
                "Version reported by the active core",
                system_info.map_or("Unknown", |info| info.library_version.as_str()),
                401,
            ),
            Self::setting(
                "Valid Extensions",
                "Content extensions accepted by the core",
                system_info.map_or("Unknown".to_string(), |info| {
                    info.valid_extensions.join(", ")
                }),
                406,
            ),
            Self::setting("Content", "Loaded content path", &content, 402),
            Self::setting(
                "Video Backend",
                "Last presented frame backend",
                backend,
                403,
            ),
            Self::setting(
                "Hardware Renderer",
                "OpenGL ES / Vulkan-MoltenVK host readiness",
                if gfx_status.hardware_ready {
                    "Ready"
                } else {
                    "Waiting for host handles"
                },
                404,
            ),
        ];
        if let Some(present) = gfx_status.last_present.as_ref() {
            entries.push(Self::setting(
                "Frame",
                "Last rendered frame counter",
                present.frame_number,
                405,
            ));
        }
        self.history.push(MenuList {
            title: "Information".to_string(),
            entries,
        });
    }

    pub fn push_status(&mut self, title: &str, message: &str) {
        self.history.push(MenuList {
            title: title.to_string(),
            entries: vec![Self::action(message, "", 0)],
        });
    }

    pub fn push_help(&mut self) {
        self.history.push(MenuList {
            title: "Help".to_string(),
            entries: vec![
                Self::setting(
                    "Accept",
                    "Open submenu or run highlighted action",
                    "A / Touch",
                    500,
                ),
                Self::setting(
                    "Cancel",
                    "Return to the previous menu list",
                    "B / Back",
                    501,
                ),
                Self::setting(
                    "Menu Model",
                    "History stack, display lists and callbacks mirror RetroArch",
                    "Rust",
                    502,
                ),
            ],
        });
    }

    pub fn push_configuration_file(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Configuration File".to_string(),
            entries: vec![
                Self::setting(
                    "Current Configuration",
                    "retroarch.cfg-compatible settings path",
                    settings.path.to_string_lossy(),
                    520,
                ),
                Self::action(
                    "Save Current Configuration",
                    "Write all managed settings",
                    521,
                ),
                Self::action(
                    "Save New Configuration",
                    "Create a new configuration file",
                    522,
                ),
                Self::action(
                    "Reset to Defaults",
                    "Restore RetroArch-compatible defaults",
                    523,
                ),
            ],
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
                ACTION_SKIN_SETTINGS,
            ),
            Self::setting(
                "XMB Theme",
                "Icon and background theme",
                &self.skin.theme,
                ACTION_SKIN_SETTINGS + 1,
            ),
            Self::setting(
                "Menu Assets",
                "XMB/Ozone asset root",
                &self.skin.assets_directory,
                ACTION_SKIN_SETTINGS + 2,
            ),
        ];
        self.history.push(MenuList {
            title: "User Interface".to_string(),
            entries,
        });
    }

    pub fn push_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Settings".to_string(),
            entries: vec![
                Self::submenu(
                    "Drivers",
                    "Video, audio, input and menu drivers",
                    ACTION_SETTINGS_DRIVERS,
                ),
                Self::submenu(
                    "Video",
                    "Scaling, filtering and synchronization",
                    ACTION_SETTINGS_VIDEO,
                ),
                Self::submenu(
                    "Audio",
                    "Audio output and synchronization",
                    ACTION_SETTINGS_AUDIO,
                ),
                Self::submenu(
                    "Input",
                    "RetroPad, overlays and autoconfig",
                    ACTION_SETTINGS_INPUT,
                ),
                Self::submenu(
                    "User Interface",
                    "Menu driver, XMB theme and assets",
                    ACTION_SETTINGS_USER_INTERFACE,
                ),
                Self::submenu(
                    "Directories",
                    "System, saves, playlists, assets and cache paths",
                    ACTION_SETTINGS_DIRECTORIES,
                ),
                Self::submenu(
                    "Saving",
                    "SaveRAM, states and runtime persistence",
                    ACTION_SETTINGS_SAVING,
                ),
                Self::submenu(
                    "Latency",
                    "Run-ahead, frame delay and hard GPU sync",
                    ACTION_SETTINGS_LATENCY,
                ),
                Self::submenu(
                    "Frame Throttle",
                    "Rewind, fast-forward and slow-motion",
                    ACTION_SETTINGS_FRAME_THROTTLE,
                ),
                Self::submenu(
                    "Playlists",
                    "History, favorites and scanned collections",
                    ACTION_SETTINGS_PLAYLISTS,
                ),
                Self::setting(
                    "Config Path",
                    "Active RetroArch-style config file",
                    settings.path.to_string_lossy(),
                    209,
                ),
            ],
        });
    }

    pub fn push_driver_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Drivers".to_string(),
            entries: vec![
                Self::setting(
                    "Video",
                    "video_driver",
                    settings.get("video_driver").map_or("bgfx", String::as_str),
                    600,
                ),
                Self::setting(
                    "Audio",
                    "audio_driver",
                    settings.get("audio_driver").map_or("swift", String::as_str),
                    601,
                ),
                Self::setting(
                    "Input",
                    "input_driver",
                    settings.get("input_driver").map_or("swift", String::as_str),
                    602,
                ),
                Self::setting(
                    "Menu",
                    "menu_driver",
                    settings.get("menu_driver").map_or("xmb", String::as_str),
                    603,
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
                Self::setting(
                    "Scaling",
                    "Integer scale / keep aspect policy",
                    "Keep Aspect",
                    621,
                ),
                Self::setting(
                    "Bilinear Filtering",
                    "Software frame interpolation",
                    "Nearest",
                    622,
                ),
                Self::setting(
                    "VSync",
                    "Synchronize presentation to display refresh",
                    "On",
                    623,
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
                Self::setting("Synchronization", "Audio sync policy", "On", 641),
                Self::setting("Latency", "Target output latency", "64 ms", 642),
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
                Self::setting(
                    "Joypad Autoconfig",
                    "Autoconfig profile directory",
                    settings
                        .path_value("joypad_autoconfig_dir")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    661,
                ),
                Self::setting(
                    "Remaps",
                    "Input remapping directory",
                    settings
                        .path_value("input_remapping_directory")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    662,
                ),
                Self::setting(
                    "Overlays",
                    "Touch overlay directory",
                    settings
                        .path_value("overlay_directory")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    663,
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
            Self::setting(
                "Screenshots",
                "Screenshot output directory",
                settings
                    .path_value("screenshot_directory")
                    .unwrap_or_default()
                    .to_string_lossy(),
                680,
            ),
            Self::setting(
                "Logs",
                "Runtime log directory",
                settings
                    .path_value("log_dir")
                    .unwrap_or_default()
                    .to_string_lossy(),
                681,
            ),
        ];
        self.history.push(MenuList {
            title: "Directories".to_string(),
            entries,
        });
    }

    pub fn push_placeholder_settings(&mut self, title: &str) {
        self.history.push(MenuList {
            title: title.to_string(),
            entries: vec![
                Self::setting(
                    "Status",
                    "Menu branch has been modeled for the UI engine",
                    "Ready",
                    700,
                ),
                Self::action(
                    "Apply Changes",
                    "Run branch-specific callback when implemented",
                    701,
                ),
            ],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_matches_retroarch_top_level_shape() {
        let engine = MenuEngine::new();
        let current = engine.current().unwrap();
        assert_eq!(current.title, "Main Menu");
        assert!(current
            .entries
            .iter()
            .any(|entry| entry.label == "Load Core"));
        assert!(current
            .entries
            .iter()
            .any(|entry| entry.label == "Load Content"));
        assert!(current
            .entries
            .iter()
            .any(|entry| entry.label == "Settings"));
        assert!(current
            .entries
            .iter()
            .any(|entry| entry.label == "Online Updater"));
    }

    #[test]
    fn settings_menu_exposes_retroarch_categories() {
        let mut engine = MenuEngine::new();
        let settings = Settings::new();
        engine.push_settings(&settings);
        let labels: Vec<&str> = engine
            .current()
            .unwrap()
            .entries
            .iter()
            .map(|entry| entry.label.as_str())
            .collect();
        for expected in [
            "Drivers",
            "Video",
            "Audio",
            "Input",
            "User Interface",
            "Directories",
            "Saving",
            "Latency",
        ] {
            assert!(labels.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn quick_menu_contains_runtime_actions() {
        let mut engine = MenuEngine::new();
        engine.push_quick_menu(true);
        let labels: Vec<&str> = engine
            .current()
            .unwrap()
            .entries
            .iter()
            .map(|entry| entry.label.as_str())
            .collect();
        for expected in [
            "Resume",
            "Restart",
            "Core Options",
            "Shaders",
            "Save States",
            "Close Content",
        ] {
            assert!(labels.contains(&expected), "missing {expected}");
        }
    }
}
