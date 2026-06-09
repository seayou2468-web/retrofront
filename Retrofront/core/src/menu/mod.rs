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
pub const ACTION_HISTORY: u32 = 21;
pub const ACTION_FAVORITES: u32 = 22;
pub const ACTION_IMPORT_CONTENT: u32 = 23;
pub const ACTION_EXPLORE: u32 = 24;
pub const ACTION_NETPLAY: u32 = 25;
pub const ACTION_RECORDING: u32 = 26;
pub const ACTION_STREAMING: u32 = 27;
pub const ACTION_ACHIEVEMENTS: u32 = 28;
pub const ACTION_CORE_CONTENTLESS: u32 = 29;
pub const ACTION_REBOOT: u32 = 30;
pub const ACTION_QUIT: u32 = 31;

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
pub const ACTION_SETTINGS_NETWORK: u32 = 220;
pub const ACTION_SETTINGS_ONSCREEN_DISPLAY: u32 = 221;
pub const ACTION_SETTINGS_ACCESSIBILITY: u32 = 222;
pub const ACTION_SETTINGS_AI_SERVICE: u32 = 223;
pub const ACTION_SETTINGS_POWER_MANAGEMENT: u32 = 224;
pub const ACTION_SETTINGS_LOGGING: u32 = 225;
pub const ACTION_SETTINGS_FILE_BROWSER: u32 = 226;
pub const ACTION_SETTINGS_RECORDING: u32 = 227;
pub const ACTION_SETTINGS_CLOUD_SYNC: u32 = 228;
pub const ACTION_SKIN_SETTINGS: u32 = 260;

pub const ACTION_DRIVER_RGUI: u32 = 270;
pub const ACTION_DRIVER_MATERIALUI: u32 = 271;
pub const ACTION_DRIVER_XMB: u32 = 272;
pub const ACTION_DRIVER_OZONE: u32 = 273;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuDriver {
    Rgui,
    MaterialUi,
    Xmb,
    Ozone,
}

impl MenuDriver {
    pub fn from_settings_id(id: &str) -> Self {
        match id.trim().to_ascii_lowercase().as_str() {
            "rgui" => Self::Rgui,
            "materialui" | "glui" => Self::MaterialUi,
            "ozone" => Self::Ozone,
            _ => Self::Xmb,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Rgui => "rgui",
            Self::MaterialUi => "materialui",
            Self::Xmb => "xmb",
            Self::Ozone => "ozone",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Rgui => "RGUI",
            Self::MaterialUi => "MaterialUI",
            Self::Xmb => "XMB",
            Self::Ozone => "Ozone",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Rgui, Self::MaterialUi, Self::Xmb, Self::Ozone]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSkin {
    pub driver: String,
    pub theme: String,
    pub assets_directory: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuDriverStyle {
    pub driver: MenuDriver,
    pub icon_family: &'static str,
    pub navigation_model: &'static str,
    pub layout_model: &'static str,
    pub supports_wallpaper: bool,
    pub supports_thumbnail_sidebar: bool,
    pub supports_touch: bool,
    pub default_theme: &'static str,
}

impl MenuDriverStyle {
    fn for_driver(driver: MenuDriver) -> Self {
        match driver {
            MenuDriver::Rgui => Self {
                driver,
                icon_family: "monospace-glyphs",
                navigation_model: "stacked-list",
                layout_model: "terminal-grid",
                supports_wallpaper: false,
                supports_thumbnail_sidebar: false,
                supports_touch: false,
                default_theme: "classic",
            },
            MenuDriver::MaterialUi => Self {
                driver,
                icon_family: "material",
                navigation_model: "drawer-and-cards",
                layout_model: "touch-list",
                supports_wallpaper: true,
                supports_thumbnail_sidebar: true,
                supports_touch: true,
                default_theme: "blue",
            },
            MenuDriver::Xmb => Self {
                driver,
                icon_family: "xmb",
                navigation_model: "horizontal-categories",
                layout_model: "ribbon",
                supports_wallpaper: true,
                supports_thumbnail_sidebar: true,
                supports_touch: false,
                default_theme: "monochrome",
            },
            MenuDriver::Ozone => Self {
                driver,
                icon_family: "ozone",
                navigation_model: "sidebar-content",
                layout_model: "desktop-panel",
                supports_wallpaper: true,
                supports_thumbnail_sidebar: true,
                supports_touch: false,
                default_theme: "nord",
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuPresentation {
    pub driver: MenuDriver,
    pub title: String,
    pub breadcrumb: Vec<String>,
    pub selected_index: usize,
    pub entry_count: usize,
    pub style: MenuDriverStyle,
}

pub struct MenuEngine {
    pub history: Vec<MenuList>,
    pub skin: MenuSkin,
    selected_indices: Vec<usize>,
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
            selected_indices: Vec::new(),
        };
        engine.push_main_menu();
        engine
    }

    pub fn driver(&self) -> MenuDriver {
        MenuDriver::from_settings_id(&self.skin.driver)
    }

    pub fn style(&self) -> MenuDriverStyle {
        MenuDriverStyle::for_driver(self.driver())
    }

    pub fn presentation(&self) -> Option<MenuPresentation> {
        self.current().map(|list| MenuPresentation {
            driver: self.driver(),
            title: list.title.clone(),
            breadcrumb: self.history.iter().map(|item| item.title.clone()).collect(),
            selected_index: *self.selected_indices.last().unwrap_or(&0),
            entry_count: list.entries.len(),
            style: self.style(),
        })
    }

    pub fn set_driver(&mut self, driver: MenuDriver) {
        self.skin.driver = driver.id().to_string();
        if self.skin.theme.is_empty()
            || self.skin.theme == "monochrome"
            || self.skin.theme == "classic"
        {
            self.skin.theme = MenuDriverStyle::for_driver(driver)
                .default_theme
                .to_string();
        }
    }

    pub fn set_selection(&mut self, index: usize) {
        if let Some(list) = self.current() {
            let max_index = list.entries.len().saturating_sub(1);
            if let Some(current) = self.selected_indices.last_mut() {
                *current = index.min(max_index);
            }
        }
    }

    pub fn push_main_menu(&mut self) {
        self.history.clear();
        self.selected_indices.clear();
        self.push_list(MenuList {
            title: "Main Menu".to_string(),
            entries: vec![
                Self::submenu("Load Core", "Select a libretro core", ACTION_LOAD_CORE),
                Self::submenu(
                    "Load Content",
                    "Browse scanned content",
                    ACTION_LOAD_CONTENT,
                ),
                Self::submenu(
                    "Contentless Cores",
                    "Start a core without content",
                    ACTION_CORE_CONTENTLESS,
                ),
                Self::submenu(
                    "Quick Menu",
                    "Runtime actions for loaded content",
                    ACTION_QUICK_MENU,
                ),
                Self::submenu("History", "Recently loaded content", ACTION_HISTORY),
                Self::submenu("Favorites", "Pinned content playlist", ACTION_FAVORITES),
                Self::submenu(
                    "Import Content",
                    "Scan files and directories into playlists",
                    ACTION_IMPORT_CONTENT,
                ),
                Self::submenu("Explore", "Filter playlists by metadata", ACTION_EXPLORE),
                Self::submenu(
                    "Netplay",
                    "Host, join and browse network sessions",
                    ACTION_NETPLAY,
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
                Self::action(
                    "Restart Retrofront",
                    "Reinitialize the frontend runtime",
                    ACTION_REBOOT,
                ),
                Self::action("Quit Retrofront", "Request frontend shutdown", ACTION_QUIT),
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
            Self::submenu(
                "Recording",
                "Record gameplay video or replay data",
                ACTION_RECORDING,
            ),
            Self::submenu(
                "Streaming",
                "Stream gameplay to a configured service",
                ACTION_STREAMING,
            ),
            Self::submenu(
                "Achievements",
                "Runtime achievement status and unlocks",
                ACTION_ACHIEVEMENTS,
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
        self.push_list(MenuList {
            title: "Quick Menu".to_string(),
            entries,
        });
    }

    pub fn push_core_list(&mut self, cores: &[CoreInfo]) {
        let entries = if cores.is_empty() {
            vec![Self::action(
                "No Cores Available",
                "Scan the configured core directory or import a libretro core",
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
        self.push_list(MenuList {
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
        self.push_list(MenuList {
            title: "Load Content".to_string(),
            entries,
        });
    }

    pub fn push_playlist_status(&mut self, title: &str, empty_message: &str) {
        self.push_list(MenuList {
            title: title.to_string(),
            entries: vec![Self::action(
                empty_message,
                "Playlist plumbing is available to the host UI",
                0,
            )],
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
            Self::setting(
                "Menu Driver",
                "Active RetroArch menu driver",
                self.driver().display_name(),
                407,
            ),
            Self::setting("Menu Theme", "Active menu theme", &self.skin.theme, 408),
        ];
        if let Some(present) = gfx_status.last_present.as_ref() {
            entries.push(Self::setting(
                "Frame",
                "Last rendered frame counter",
                present.frame_number,
                405,
            ));
        }
        self.push_list(MenuList {
            title: "Information".to_string(),
            entries,
        });
    }

    pub fn push_status(&mut self, title: &str, message: &str) {
        self.push_list(MenuList {
            title: title.to_string(),
            entries: vec![Self::action(message, "", 0)],
        });
    }

    pub fn push_help(&mut self) {
        self.push_list(MenuList {
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
                    "Search",
                    "Filter display-list entries when the host UI supports text input",
                    "/",
                    503,
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
        self.push_list(MenuList {
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
        let driver =
            MenuDriver::from_settings_id(settings.get("menu_driver").map_or("xmb", String::as_str));
        self.skin = MenuSkin {
            driver: driver.id().to_string(),
            theme: settings.get("menu_xmb_theme").cloned().unwrap_or_else(|| {
                MenuDriverStyle::for_driver(driver)
                    .default_theme
                    .to_string()
            }),
            assets_directory: settings
                .menu_assets_directory()
                .to_string_lossy()
                .into_owned(),
        };
    }

    pub fn push_skin_settings(&mut self, settings: &Settings) {
        self.apply_skin_from_settings(settings);
        let style = self.style();
        let mut entries = vec![
            Self::setting(
                "Menu Driver",
                "RetroArch-compatible menu driver id",
                &self.skin.driver,
                ACTION_SKIN_SETTINGS,
            ),
            Self::setting(
                "Theme",
                "Icon and background theme",
                &self.skin.theme,
                ACTION_SKIN_SETTINGS + 1,
            ),
            Self::setting(
                "Menu Assets",
                "RGUI/MaterialUI/XMB/Ozone asset root",
                &self.skin.assets_directory,
                ACTION_SKIN_SETTINGS + 2,
            ),
            Self::setting(
                "Layout",
                "Driver-specific layout model",
                style.layout_model,
                ACTION_SKIN_SETTINGS + 3,
            ),
            Self::setting(
                "Navigation",
                "Driver-specific navigation model",
                style.navigation_model,
                ACTION_SKIN_SETTINGS + 4,
            ),
            Self::setting(
                "Icons",
                "Driver-specific icon family",
                style.icon_family,
                ACTION_SKIN_SETTINGS + 5,
            ),
            Self::setting(
                "Wallpaper",
                "Background image support",
                if style.supports_wallpaper {
                    "On"
                } else {
                    "Off"
                },
                ACTION_SKIN_SETTINGS + 6,
            ),
            Self::setting(
                "Touch",
                "Touch-first menu operation",
                if style.supports_touch { "On" } else { "Off" },
                ACTION_SKIN_SETTINGS + 7,
            ),
        ];
        for driver in MenuDriver::all() {
            entries.push(Self::action(
                driver.display_name(),
                "Switch active RetroArch menu driver",
                match driver {
                    MenuDriver::Rgui => ACTION_DRIVER_RGUI,
                    MenuDriver::MaterialUi => ACTION_DRIVER_MATERIALUI,
                    MenuDriver::Xmb => ACTION_DRIVER_XMB,
                    MenuDriver::Ozone => ACTION_DRIVER_OZONE,
                },
            ));
        }
        self.push_list(MenuList {
            title: "User Interface".to_string(),
            entries,
        });
    }

    pub fn push_settings(&mut self, settings: &Settings) {
        self.push_list(MenuList {
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
                    "Menu driver, theme and assets",
                    ACTION_SETTINGS_USER_INTERFACE,
                ),
                Self::submenu(
                    "On-Screen Display",
                    "Overlays, notifications and widgets",
                    ACTION_SETTINGS_ONSCREEN_DISPLAY,
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
                Self::submenu(
                    "Network",
                    "Netplay, updater and connectivity options",
                    ACTION_SETTINGS_NETWORK,
                ),
                Self::submenu(
                    "Accessibility",
                    "Text-to-speech and menu accessibility",
                    ACTION_SETTINGS_ACCESSIBILITY,
                ),
                Self::submenu(
                    "AI Service",
                    "Translation and image service hooks",
                    ACTION_SETTINGS_AI_SERVICE,
                ),
                Self::submenu(
                    "Power Management",
                    "Battery, screensaver and power hints",
                    ACTION_SETTINGS_POWER_MANAGEMENT,
                ),
                Self::submenu(
                    "Logging",
                    "Runtime logging verbosity and output",
                    ACTION_SETTINGS_LOGGING,
                ),
                Self::submenu(
                    "File Browser",
                    "Browser filtering and behavior",
                    ACTION_SETTINGS_FILE_BROWSER,
                ),
                Self::submenu(
                    "Recording",
                    "Video, replay and streaming settings",
                    ACTION_SETTINGS_RECORDING,
                ),
                Self::submenu(
                    "Cloud Sync",
                    "Remote save synchronization",
                    ACTION_SETTINGS_CLOUD_SYNC,
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
        self.push_list(MenuList {
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
                Self::setting(
                    "Camera",
                    "camera_driver",
                    settings.get("camera_driver").map_or("null", String::as_str),
                    604,
                ),
                Self::setting(
                    "Location",
                    "location_driver",
                    settings
                        .get("location_driver")
                        .map_or("null", String::as_str),
                    605,
                ),
                Self::setting(
                    "Bluetooth",
                    "bluetooth_driver",
                    settings
                        .get("bluetooth_driver")
                        .map_or("null", String::as_str),
                    606,
                ),
                Self::setting(
                    "Wi-Fi",
                    "wifi_driver",
                    settings.get("wifi_driver").map_or("null", String::as_str),
                    607,
                ),
            ],
        });
    }

    pub fn push_video_settings(&mut self, settings: &Settings) {
        self.push_list(MenuList {
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
                    settings
                        .get("video_scale_mode")
                        .map_or("Keep Aspect", String::as_str),
                    621,
                ),
                Self::setting(
                    "Bilinear Filtering",
                    "Software frame interpolation",
                    settings
                        .get("video_smooth")
                        .map_or("Nearest", String::as_str),
                    622,
                ),
                Self::setting(
                    "VSync",
                    "Synchronize presentation to display refresh",
                    settings.get("video_vsync").map_or("On", String::as_str),
                    623,
                ),
                Self::setting(
                    "Threaded Video",
                    "Decouple video presentation from emulation",
                    settings.get("video_threaded").map_or("Off", String::as_str),
                    624,
                ),
                Self::setting(
                    "Hard GPU Sync",
                    "Reduce latency by synchronizing GPU work",
                    settings
                        .get("video_hard_sync")
                        .map_or("Off", String::as_str),
                    625,
                ),
            ],
        });
    }

    pub fn push_audio_settings(&mut self, settings: &Settings) {
        self.push_list(MenuList {
            title: "Audio".to_string(),
            entries: vec![
                Self::setting(
                    "Output",
                    "Active audio driver",
                    settings.get("audio_driver").map_or("swift", String::as_str),
                    640,
                ),
                Self::setting(
                    "Synchronization",
                    "Audio sync policy",
                    settings.get("audio_sync").map_or("On", String::as_str),
                    641,
                ),
                Self::setting(
                    "Latency",
                    "Target output latency",
                    settings
                        .get("audio_latency")
                        .map_or("64 ms", String::as_str),
                    642,
                ),
                Self::setting(
                    "Mute",
                    "Global audio mute",
                    settings.get("audio_mute").map_or("Off", String::as_str),
                    643,
                ),
                Self::setting(
                    "DSP Plugin",
                    "Audio DSP filter path",
                    settings
                        .get("audio_dsp_plugin")
                        .map_or("None", String::as_str),
                    644,
                ),
            ],
        });
    }

    pub fn push_input_settings(&mut self, settings: &Settings) {
        self.push_list(MenuList {
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
                        .get("joypad_autoconfig_dir")
                        .map_or("autoconfig", String::as_str),
                    661,
                ),
                Self::setting(
                    "Remaps",
                    "Input remap directory",
                    settings
                        .get("input_remapping_directory")
                        .map_or("remaps", String::as_str),
                    662,
                ),
                Self::setting(
                    "Hotkeys",
                    "Menu toggle, quit, save/load and fast-forward bindings",
                    "RetroPad",
                    663,
                ),
                Self::setting(
                    "Turbo",
                    "Turbo-fire settings",
                    settings
                        .get("input_turbo_period")
                        .map_or("Default", String::as_str),
                    664,
                ),
                Self::setting(
                    "Overlays",
                    "Touch/gamepad overlay directory",
                    settings
                        .get("overlay_directory")
                        .map_or("overlays", String::as_str),
                    665,
                ),
            ],
        });
    }

    pub fn push_directory_settings(&mut self, settings: &Settings) {
        let entries = vec![
            Self::setting(
                "Core Directory",
                "libretro dynamic libraries",
                settings.libretro_directory().to_string_lossy(),
                200,
            ),
            Self::setting(
                "Core Info Directory",
                ".info metadata used for core matching",
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
            Self::setting(
                "Cheats",
                "Cheat database path",
                settings
                    .path_value("cheat_database_path")
                    .unwrap_or_default()
                    .to_string_lossy(),
                682,
            ),
            Self::setting(
                "Overlays",
                "On-screen overlay directory",
                settings
                    .path_value("overlay_directory")
                    .unwrap_or_default()
                    .to_string_lossy(),
                683,
            ),
        ];
        self.push_list(MenuList {
            title: "Directories".to_string(),
            entries,
        });
    }

    pub fn push_setting_group(&mut self, title: &str, entries: Vec<MenuEntry>) {
        self.push_list(MenuList {
            title: title.to_string(),
            entries,
        });
    }

    pub fn push_placeholder_settings(&mut self, title: &str) {
        self.push_setting_group(
            title,
            vec![
                Self::setting(
                    "Enable",
                    "RetroArch-compatible branch is available to the UI",
                    "Ready",
                    700,
                ),
                Self::setting(
                    "State",
                    "Runtime callback can be connected by the host frontend",
                    "Modeled",
                    701,
                ),
                Self::action(
                    "Apply Changes",
                    "Run branch-specific callback when implemented",
                    702,
                ),
            ],
        );
    }

    pub fn pop(&mut self) -> Option<MenuList> {
        if self.history.len() > 1 {
            self.selected_indices.pop();
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
        self.selected_indices.truncate(1);
    }

    fn push_list(&mut self, list: MenuList) {
        self.history.push(list);
        self.selected_indices.push(0);
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
        for expected in [
            "Load Core",
            "Load Content",
            "Settings",
            "Online Updater",
            "History",
            "Favorites",
            "Import Content",
            "Explore",
            "Netplay",
        ] {
            assert!(
                current.entries.iter().any(|entry| entry.label == expected),
                "missing {expected}"
            );
        }
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
            "Network",
            "On-Screen Display",
            "Accessibility",
            "Logging",
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
            "Recording",
            "Streaming",
            "Achievements",
        ] {
            assert!(labels.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn all_retroarch_menu_drivers_have_styles() {
        for driver in MenuDriver::all() {
            let style = MenuDriverStyle::for_driver(*driver);
            assert_eq!(style.driver, *driver);
            assert!(!style.icon_family.is_empty());
            assert!(!style.layout_model.is_empty());
        }
    }

    #[test]
    fn presentation_tracks_breadcrumb_and_selection() {
        let mut engine = MenuEngine::new();
        engine.push_help();
        engine.set_selection(2);
        let presentation = engine.presentation().unwrap();
        assert_eq!(presentation.driver, MenuDriver::Xmb);
        assert_eq!(presentation.breadcrumb, vec!["Main Menu", "Help"]);
        assert_eq!(presentation.selected_index, 2);
    }
}
