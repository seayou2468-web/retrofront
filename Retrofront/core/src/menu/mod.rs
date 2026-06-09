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
pub const ACTION_CORE_SETTINGS: u32 = 21;
pub const ACTION_DISPLAY_SETTINGS: u32 = 22;
pub const ACTION_AUDIO_MIXER: u32 = 23;
pub const ACTION_DISC_CONTROL: u32 = 24;
pub const ACTION_INPUT_MAPPING: u32 = 25;
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
pub const ACTION_SETTINGS_PLAY_SCREEN: u32 = 220;
pub const ACTION_SETTINGS_LIBRARY: u32 = 221;
pub const ACTION_SETTINGS_CORE: u32 = 222;
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
                driver: "oneui".to_string(),
                theme: "dark".to_string(),
                assets_directory: String::new(),
            },
        };
        engine.push_main_menu();
        engine
    }

    pub fn push_main_menu(&mut self) {
        self.history.clear();
        self.history.push(MenuList {
            title: "Home".to_string(),
            entries: vec![
                Self::submenu(
                    "Library",
                    "ROM-only library with direct compatible-core launch",
                    ACTION_LOAD_CONTENT,
                ),
                Self::submenu(
                    "Cores",
                    "Manage and select libretro cores",
                    ACTION_LOAD_CORE,
                ),
                Self::submenu(
                    "Quick Menu",
                    "In-game actions, display, controls and core settings",
                    ACTION_QUICK_MENU,
                ),
                Self::submenu(
                    "Settings",
                    "One UI, play screen, library, drivers and paths",
                    ACTION_SETTINGS,
                ),
                Self::submenu(
                    "Information",
                    "Core, content and runtime metadata",
                    ACTION_INFORMATION,
                ),
            ],
        });
    }

    pub fn push_quick_menu(&mut self, has_game: bool) {
        let mut entries = vec![
            Self::action(
                "Resume",
                "Return to the running game",
                ACTION_RESUME_CONTENT,
            ),
            Self::action(
                "Restart",
                "Reset the current content",
                ACTION_RESTART_CONTENT,
            ),
            Self::submenu(
                "Core Settings",
                "Open options for the active libretro core",
                ACTION_CORE_SETTINGS,
            ),
            Self::submenu(
                "Display & Orientation",
                "Portrait, landscape, scaling and filter controls",
                ACTION_DISPLAY_SETTINGS,
            ),
            Self::submenu(
                "Controls",
                "Button mapping, remaps, overlays and connected pads",
                ACTION_CONTROLS,
            ),
            Self::submenu(
                "Save States",
                "Save, load and manage state slots",
                ACTION_SAVE_STATES,
            ),
            Self::submenu("Shaders", "Video shader passes and presets", ACTION_SHADERS),
            Self::submenu("Cheats", "Load and apply cheat files", ACTION_CHEATS),
            Self::submenu(
                "Overrides",
                "Core/content/game override configuration",
                ACTION_OVERRIDES,
            ),
            Self::submenu(
                "Audio",
                "Mixer, mute and latency shortcuts",
                ACTION_AUDIO_MIXER,
            ),
            Self::submenu(
                "Disc Control",
                "Swap/eject virtual discs when a core supports it",
                ACTION_DISC_CONTROL,
            ),
            Self::action(
                "Screenshot",
                "Write a screenshot to the screenshots directory",
                ACTION_TAKE_SCREENSHOT,
            ),
            Self::action(
                "Favorite",
                "Add current ROM to Favorites",
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
                "Scan the bundled/configured core directories",
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
            title: "Cores".to_string(),
            entries,
        });
    }

    pub fn push_content_list(&mut self, games: &[GameEntry]) {
        let entries = if games.is_empty() {
            vec![Self::action(
                "No ROMs Found",
                "Scan the ROM directory; non-ROM files are excluded",
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
            title: "Library".to_string(),
            entries,
        });
    }

    pub fn push_core_choice(&mut self, content_label: &str, cores: &[CoreInfo]) {
        let entries = if cores.is_empty() {
            vec![Self::action(
                "No Compatible Cores",
                "Install or scan a core that supports this ROM",
                0,
            )]
        } else {
            cores
                .iter()
                .enumerate()
                .map(|(i, core)| MenuEntry {
                    label: core.display_name.clone(),
                    sublabel: if core.system_name.is_empty() {
                        core.supported_extensions.join(", ")
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
            title: format!("Choose Core for {content_label}"),
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
            title: "Controls Help".to_string(),
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
            driver: "oneui".to_string(),
            theme: settings
                .get("menu_theme")
                .cloned()
                .unwrap_or_else(|| "dark".to_string()),
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
                "Menu Engine",
                "Locked to One UI; legacy engines are not exposed",
                &self.skin.driver,
                ACTION_SKIN_SETTINGS,
            ),
            Self::setting(
                "Theme",
                "Dark One UI color theme",
                &self.skin.theme,
                ACTION_SKIN_SETTINGS + 1,
            ),
            Self::setting(
                "Skin Assets",
                "One UI skin asset root",
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
                    "One UI & Skin",
                    "Dark mode, density, cards and skin assets",
                    ACTION_SETTINGS_USER_INTERFACE,
                ),
                Self::submenu(
                    "Play Screen",
                    "Portrait, landscape, scaling and quick menu behavior",
                    ACTION_SETTINGS_PLAY_SCREEN,
                ),
                Self::submenu(
                    "Library",
                    "ROM-only scanning, playlists and thumbnails",
                    ACTION_SETTINGS_LIBRARY,
                ),
                Self::submenu(
                    "Core",
                    "Core options, BIOS paths and compatibility behavior",
                    ACTION_SETTINGS_CORE,
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
                    "ROM playlists, history and favorites",
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
                    settings.get("video_driver").map_or("metal", String::as_str),
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
                    settings.get("menu_driver").map_or("oneui", String::as_str),
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
                    settings.get("video_driver").map_or("metal", String::as_str),
                    620,
                ),
                Self::setting(
                    "Scaling",
                    "Integer scale / keep aspect policy",
                    settings
                        .get("video_scale_mode")
                        .map_or("keep_aspect", String::as_str),
                    621,
                ),
                Self::setting(
                    "Bilinear Filtering",
                    "Software frame interpolation",
                    settings
                        .get("video_filter_mode")
                        .map_or("nearest", String::as_str),
                    622,
                ),
                Self::setting(
                    "VSync",
                    "Synchronize presentation to display refresh",
                    settings.get("video_vsync").map_or("true", String::as_str),
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
                Self::setting(
                    "Enabled",
                    "Master audio output",
                    settings.get("audio_enable").map_or("true", String::as_str),
                    641,
                ),
                Self::setting(
                    "Synchronization",
                    "Audio sync policy",
                    settings.get("audio_sync").map_or("true", String::as_str),
                    642,
                ),
                Self::setting(
                    "Latency",
                    "Target output latency in milliseconds",
                    settings
                        .get("audio_latency_ms")
                        .map_or("64", String::as_str),
                    643,
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
                Self::setting(
                    "Overlay Enabled",
                    "Show touch overlay while playing",
                    settings
                        .get("input_overlay_enable")
                        .map_or("true", String::as_str),
                    664,
                ),
                Self::setting(
                    "Haptics",
                    "Touch feedback for virtual controls",
                    settings
                        .get("input_haptic_feedback")
                        .map_or("true", String::as_str),
                    665,
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

    pub fn push_play_screen_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Play Screen".to_string(),
            entries: vec![
                Self::setting(
                    "Orientation",
                    "Auto, portrait or landscape",
                    settings
                        .get("play_screen_orientation")
                        .map_or("auto", String::as_str),
                    690,
                ),
                Self::setting(
                    "Portrait Layout",
                    "Fit screen controls without oversized buttons",
                    settings
                        .get("play_screen_portrait_layout")
                        .map_or("fit", String::as_str),
                    691,
                ),
                Self::setting(
                    "Landscape Layout",
                    "Immersive landscape viewport",
                    settings
                        .get("play_screen_landscape_layout")
                        .map_or("immersive", String::as_str),
                    692,
                ),
                Self::setting(
                    "Quick Menu",
                    "One UI bottom-sheet style",
                    settings
                        .get("quick_menu_style")
                        .map_or("oneui_sheet", String::as_str),
                    693,
                ),
                Self::setting(
                    "Scale Mode",
                    "Keep aspect / integer / stretch",
                    settings
                        .get("video_scale_mode")
                        .map_or("keep_aspect", String::as_str),
                    694,
                ),
            ],
        });
    }

    pub fn push_library_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Library".to_string(),
            entries: vec![
                Self::setting(
                    "Mode",
                    "Only ROM-compatible files are listed",
                    settings
                        .get("library_mode")
                        .map_or("roms_only", String::as_str),
                    710,
                ),
                Self::setting(
                    "ROM Directory",
                    "Library scan root",
                    settings.content_directory().to_string_lossy(),
                    711,
                ),
                Self::setting(
                    "Playlists",
                    "ROM history and favorites",
                    settings
                        .path_value("playlist_directory")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    712,
                ),
                Self::setting(
                    "Thumbnails",
                    "Box art and screenshots",
                    settings.thumbnails_directory().to_string_lossy(),
                    713,
                ),
                Self::setting(
                    "Core Choice",
                    "Ask when multiple compatible cores exist",
                    "On",
                    714,
                ),
                Self::setting(
                    "Sort",
                    "Library ordering mode",
                    settings
                        .get("library_sort_mode")
                        .map_or("name_ascending", String::as_str),
                    715,
                ),
                Self::setting(
                    "Core Badges",
                    "Show compatible core counts on ROM rows",
                    settings
                        .get("library_show_core_badges")
                        .map_or("true", String::as_str),
                    716,
                ),
                Self::setting(
                    "File Details",
                    "Show ROM extension and file size",
                    settings
                        .get("library_show_file_details")
                        .map_or("true", String::as_str),
                    717,
                ),
                Self::setting(
                    "Auto Scan",
                    "Refresh ROM list on app launch",
                    settings
                        .get("library_auto_scan_on_launch")
                        .map_or("true", String::as_str),
                    718,
                ),
            ],
        });
    }

    pub fn push_core_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Core".to_string(),
            entries: vec![
                Self::setting(
                    "Core Settings",
                    "Variables exposed by the active core",
                    settings
                        .path_value("core_options_path")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    720,
                ),
                Self::setting(
                    "System/BIOS",
                    "Firmware directory",
                    settings.system_directory().to_string_lossy(),
                    721,
                ),
                Self::setting(
                    "SaveRAM",
                    "Battery-backed saves",
                    settings.savefile_directory().to_string_lossy(),
                    722,
                ),
                Self::setting(
                    "States",
                    "Instant save states",
                    settings.savestate_directory().to_string_lossy(),
                    723,
                ),
                Self::setting(
                    "Preferred Core",
                    "Remember per-extension core selections",
                    "On",
                    724,
                ),
                Self::setting(
                    "Auto Save State",
                    "Save a state when closing content",
                    settings
                        .get("savestate_auto_save")
                        .map_or("false", String::as_str),
                    725,
                ),
                Self::setting(
                    "Auto Load State",
                    "Load the latest state when launching content",
                    settings
                        .get("savestate_auto_load")
                        .map_or("false", String::as_str),
                    726,
                ),
            ],
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
        assert_eq!(current.title, "Home");
        assert!(current.entries.iter().any(|entry| entry.label == "Cores"));
        assert!(current.entries.iter().any(|entry| entry.label == "Library"));
        assert!(current
            .entries
            .iter()
            .any(|entry| entry.label == "Settings"));
        assert!(current
            .entries
            .iter()
            .any(|entry| entry.label == "Quick Menu"));
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
            "One UI & Skin",
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
            "Core Settings",
            "Shaders",
            "Save States",
            "Close Content",
        ] {
            assert!(labels.contains(&expected), "missing {expected}");
        }
    }
}
