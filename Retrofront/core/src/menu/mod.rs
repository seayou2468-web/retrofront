use crate::core_info::CoreInfo;
use crate::gfx::{GfxBackendKind, GfxStatus};
use crate::scanner::GameEntry;
use crate::settings::Settings;
use crate::{GameInfo, SystemInfo};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uint, c_void};
use std::path::Path;

#[repr(C)]
struct NativeMenuDriverSpec {
    ident: *const c_char,
    display_name: *const c_char,
    layout_model: *const c_char,
    input_model: *const c_char,
    thumbnail_model: *const c_char,
    animation_model: *const c_char,
    asset_subdirectory: *const c_char,
    default_theme: *const c_char,
}

#[repr(C)]
struct NativeMenuHostCallbacks {
    get_setting: Option<extern "C" fn(*const c_char, *mut c_void) -> *const c_char>,
    set_setting: Option<extern "C" fn(*const c_char, *const c_char, *mut c_void) -> c_uint>,
    directory_exists: Option<extern "C" fn(*const c_char, *mut c_void) -> c_uint>,
    file_exists: Option<extern "C" fn(*const c_char, *mut c_void) -> c_uint>,
    userdata: *mut c_void,
}

#[repr(C)]
struct NativeMenuSourceFile {
    path: *const c_char,
    compiled: c_uint,
}

#[repr(C)]
struct NativeMenuLayoutMetrics {
    viewport_width: c_uint,
    viewport_height: c_uint,
    content_x: c_uint,
    content_y: c_uint,
    content_width: c_uint,
    content_height: c_uint,
    sidebar_width: c_uint,
    header_height: c_uint,
    footer_height: c_uint,
    row_height: c_uint,
    icon_size: c_uint,
    horizontal_padding: c_uint,
    vertical_padding: c_uint,
    background_mode: c_uint,
    scale: f32,
}

#[repr(C)]
struct NativeMenuRuntimeConfig {
    driver: *const NativeMenuDriverSpec,
    driver_ident: *const c_char,
    assets_directory: *const c_char,
    theme: *const c_char,
    assets_ready: c_uint,
}

#[repr(C)]
struct NativeMenuResolvedAssets {
    root_directory: *const c_char,
    driver_directory: *const c_char,
    icon_directory: *const c_char,
    font_path: *const c_char,
    background_path: *const c_char,
    assets_ready: c_uint,
}

extern "C" {
    fn rf_menu_source_file_count() -> c_uint;
    fn rf_menu_source_file_at(index: c_uint) -> *const NativeMenuSourceFile;
    fn rf_menu_layout_for_viewport(
        driver_ident: *const c_char,
        viewport_width: c_uint,
        viewport_height: c_uint,
        out_metrics: *mut NativeMenuLayoutMetrics,
    ) -> c_uint;
    fn rf_menu_driver_at(index: c_uint) -> *const NativeMenuDriverSpec;
    fn rf_menu_driver_default() -> *const NativeMenuDriverSpec;
    fn rf_menu_driver_next_ident(ident: *const c_char) -> *const c_char;
    fn rf_menu_connect_host(callbacks: *const NativeMenuHostCallbacks);
    fn rf_menu_get_runtime_config(out_config: *mut NativeMenuRuntimeConfig) -> c_uint;
    fn rf_menu_resolve_assets(
        driver_ident: *const c_char,
        out_assets: *mut NativeMenuResolvedAssets,
    ) -> c_uint;
}

#[derive(Debug)]
pub struct NativeMenuBridge {
    settings: Vec<(CString, CString)>,
}

impl NativeMenuBridge {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            settings: settings
                .values
                .iter()
                .map(|(key, value)| {
                    (
                        CString::new(key.as_str()).unwrap_or_default(),
                        CString::new(value.as_str()).unwrap_or_default(),
                    )
                })
                .collect(),
        }
    }

    pub fn sync_runtime_config(&mut self) -> NativeMenuRuntimeSnapshot {
        let callbacks = NativeMenuHostCallbacks {
            get_setting: Some(native_bridge_get_setting),
            set_setting: Some(native_bridge_set_setting),
            directory_exists: Some(native_bridge_directory_exists),
            file_exists: Some(native_bridge_file_exists),
            userdata: (self as *mut Self).cast(),
        };
        unsafe { rf_menu_connect_host(&callbacks) };
        let mut raw = NativeMenuRuntimeConfig {
            driver: std::ptr::null(),
            driver_ident: std::ptr::null(),
            assets_directory: std::ptr::null(),
            theme: std::ptr::null(),
            assets_ready: 0,
        };
        let ok = unsafe { rf_menu_get_runtime_config(&mut raw) } != 0;
        unsafe { rf_menu_connect_host(std::ptr::null()) };
        if !ok {
            return NativeMenuRuntimeSnapshot::default();
        }
        NativeMenuRuntimeSnapshot {
            driver: native_str(raw.driver_ident, "materialui").to_string(),
            theme: native_str(raw.theme, "dark").to_string(),
            assets_directory: native_str(raw.assets_directory, "").to_string(),
            assets_ready: raw.assets_ready != 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMenuRuntimeSnapshot {
    pub driver: String,
    pub theme: String,
    pub assets_directory: String,
    pub assets_ready: bool,
}

impl Default for NativeMenuRuntimeSnapshot {
    fn default() -> Self {
        Self {
            driver: "materialui".to_string(),
            theme: "dark".to_string(),
            assets_directory: String::new(),
            assets_ready: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NativeMenuAssetsSnapshot {
    pub root_directory: String,
    pub driver_directory: String,
    pub icon_directory: String,
    pub font_path: String,
    pub background_path: String,
    pub assets_ready: bool,
}

impl NativeMenuBridge {
    pub fn resolve_assets(&mut self, driver: &str) -> NativeMenuAssetsSnapshot {
        let callbacks = NativeMenuHostCallbacks {
            get_setting: Some(native_bridge_get_setting),
            set_setting: Some(native_bridge_set_setting),
            directory_exists: Some(native_bridge_directory_exists),
            file_exists: Some(native_bridge_file_exists),
            userdata: (self as *mut Self).cast(),
        };
        unsafe { rf_menu_connect_host(&callbacks) };
        let driver = CString::new(driver).unwrap_or_default();
        let mut raw = NativeMenuResolvedAssets {
            root_directory: std::ptr::null(),
            driver_directory: std::ptr::null(),
            icon_directory: std::ptr::null(),
            font_path: std::ptr::null(),
            background_path: std::ptr::null(),
            assets_ready: 0,
        };
        let ok = unsafe { rf_menu_resolve_assets(driver.as_ptr(), &mut raw) } != 0;
        unsafe { rf_menu_connect_host(std::ptr::null()) };
        if !ok {
            return NativeMenuAssetsSnapshot::default();
        }
        NativeMenuAssetsSnapshot {
            root_directory: c_str_lossy(raw.root_directory).unwrap_or_default(),
            driver_directory: c_str_lossy(raw.driver_directory).unwrap_or_default(),
            icon_directory: c_str_lossy(raw.icon_directory).unwrap_or_default(),
            font_path: c_str_lossy(raw.font_path).unwrap_or_default(),
            background_path: c_str_lossy(raw.background_path).unwrap_or_default(),
            assets_ready: raw.assets_ready != 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuLayoutMetrics {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub content_x: u32,
    pub content_y: u32,
    pub content_width: u32,
    pub content_height: u32,
    pub sidebar_width: u32,
    pub header_height: u32,
    pub footer_height: u32,
    pub row_height: u32,
    pub icon_size: u32,
    pub horizontal_padding: u32,
    pub vertical_padding: u32,
    pub background_mode: u32,
    pub scale: f32,
}

pub fn native_menu_source_files() -> Vec<(String, bool)> {
    let count = unsafe { rf_menu_source_file_count() };
    (0..count)
        .filter_map(|index| {
            let raw = unsafe { rf_menu_source_file_at(index) };
            let raw = unsafe { raw.as_ref() }?;
            Some((c_str_lossy(raw.path)?, raw.compiled != 0))
        })
        .collect()
}

pub fn native_menu_layout(driver: &str, width: u32, height: u32) -> Option<MenuLayoutMetrics> {
    let driver = CString::new(driver).unwrap_or_default();
    let mut raw = NativeMenuLayoutMetrics {
        viewport_width: 0,
        viewport_height: 0,
        content_x: 0,
        content_y: 0,
        content_width: 0,
        content_height: 0,
        sidebar_width: 0,
        header_height: 0,
        footer_height: 0,
        row_height: 0,
        icon_size: 0,
        horizontal_padding: 0,
        vertical_padding: 0,
        background_mode: 0,
        scale: 1.0,
    };
    let ok = unsafe { rf_menu_layout_for_viewport(driver.as_ptr(), width, height, &mut raw) } != 0;
    ok.then_some(MenuLayoutMetrics {
        viewport_width: raw.viewport_width,
        viewport_height: raw.viewport_height,
        content_x: raw.content_x,
        content_y: raw.content_y,
        content_width: raw.content_width,
        content_height: raw.content_height,
        sidebar_width: raw.sidebar_width,
        header_height: raw.header_height,
        footer_height: raw.footer_height,
        row_height: raw.row_height,
        icon_size: raw.icon_size,
        horizontal_padding: raw.horizontal_padding,
        vertical_padding: raw.vertical_padding,
        background_mode: raw.background_mode,
        scale: raw.scale,
    })
}

extern "C" fn native_bridge_get_setting(
    key: *const c_char,
    userdata: *mut c_void,
) -> *const c_char {
    let Some(bridge) = (unsafe { (userdata as *mut NativeMenuBridge).as_ref() }) else {
        return std::ptr::null();
    };
    let Some(key) = c_str_lossy(key) else {
        return std::ptr::null();
    };
    bridge
        .settings
        .iter()
        .find(|(stored_key, _)| stored_key.as_c_str().to_bytes() == key.as_bytes())
        .map_or(std::ptr::null(), |(_, value)| value.as_ptr())
}

extern "C" fn native_bridge_set_setting(
    key: *const c_char,
    value: *const c_char,
    userdata: *mut c_void,
) -> c_uint {
    let Some(bridge) = (unsafe { (userdata as *mut NativeMenuBridge).as_mut() }) else {
        return 0;
    };
    let Some(key) = c_str_lossy(key) else {
        return 0;
    };
    let Some(value) = c_str_lossy(value) else {
        return 0;
    };
    let key_c = CString::new(key.as_bytes()).unwrap_or_default();
    let value_c = CString::new(value.as_bytes()).unwrap_or_default();
    if let Some((_, stored_value)) = bridge
        .settings
        .iter_mut()
        .find(|(stored_key, _)| stored_key.as_c_str().to_bytes() == key_c.as_c_str().to_bytes())
    {
        *stored_value = value_c;
    } else {
        bridge.settings.push((key_c, value_c));
    }
    1
}

extern "C" fn native_bridge_directory_exists(
    path: *const c_char,
    _userdata: *mut c_void,
) -> c_uint {
    let Some(path) = c_str_lossy(path) else {
        return 0;
    };
    Path::new(&path).is_dir() as c_uint
}

extern "C" fn native_bridge_file_exists(path: *const c_char, _userdata: *mut c_void) -> c_uint {
    let Some(path) = c_str_lossy(path) else {
        return 0;
    };
    Path::new(&path).is_file() as c_uint
}

fn c_str_lossy(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned(),
    )
}

fn native_str(ptr: *const c_char, fallback: &'static str) -> &'static str {
    if ptr.is_null() {
        return fallback;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or(fallback)
}

fn native_spec(ptr: *const NativeMenuDriverSpec) -> MenuDriverSpec {
    let ptr = if ptr.is_null() {
        unsafe { rf_menu_driver_default() }
    } else {
        ptr
    };
    if ptr.is_null() {
        return MenuDriverSpec::materialui();
    }
    let spec = unsafe { &*ptr };
    MenuDriverSpec {
        ident: native_str(spec.ident, "materialui"),
        display_name: native_str(spec.display_name, "Material UI"),
        layout_model: native_str(spec.layout_model, "mobile_appbar_navigation"),
        input_model: native_str(spec.input_model, "touch_navigation_retropad"),
        thumbnail_model: native_str(spec.thumbnail_model, "responsive_dual_thumbnail_list"),
        animation_model: native_str(spec.animation_model, "material_elevation_ripple"),
        asset_subdirectory: native_str(spec.asset_subdirectory, "materialui"),
        default_theme: native_str(spec.default_theme, "dark"),
    }
}

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
pub const ACTION_EXIT_GAME: u32 = 26;
pub const ACTION_SAVE_STATE_SLOT_0: u32 = 27;
pub const ACTION_LOAD_STATE_SLOT_0: u32 = 28;
pub const ACTION_SAVE_SRAM: u32 = 29;
pub const ACTION_STATE_SLOT: u32 = 30;
pub const ACTION_STATE_SLOT_DECREASE: u32 = 38;
pub const ACTION_STATE_SLOT_INCREASE: u32 = 39;
pub const ACTION_UNDO_LOAD_STATE: u32 = 31;
pub const ACTION_UNDO_SAVE_STATE: u32 = 32;
pub const ACTION_REPLAY: u32 = 33;
pub const ACTION_RECORDING: u32 = 34;
pub const ACTION_STREAMING: u32 = 35;
pub const ACTION_ADD_TO_PLAYLIST: u32 = 36;
pub const ACTION_SET_CORE_ASSOCIATION: u32 = 37;
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
pub enum MenuDriver {
    Ozone,
    MaterialUi,
    Rgui,
    Xmb,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuDriverSpec {
    pub ident: &'static str,
    pub display_name: &'static str,
    pub layout_model: &'static str,
    pub input_model: &'static str,
    pub thumbnail_model: &'static str,
    pub animation_model: &'static str,
    pub asset_subdirectory: &'static str,
    pub default_theme: &'static str,
}

impl MenuDriverSpec {
    const fn materialui() -> Self {
        Self {
            ident: "materialui",
            display_name: "Material UI",
            layout_model: "mobile_appbar_navigation",
            input_model: "touch_navigation_retropad",
            thumbnail_model: "responsive_dual_thumbnail_list",
            animation_model: "material_elevation_ripple",
            asset_subdirectory: "materialui",
            default_theme: "dark",
        }
    }
}

impl MenuDriver {
    pub const ALL: [MenuDriver; 4] = [
        MenuDriver::MaterialUi,
        MenuDriver::Ozone,
        MenuDriver::Xmb,
        MenuDriver::Rgui,
    ];

    pub fn from_ident(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "ozone" => MenuDriver::Ozone,
            "rgui" => MenuDriver::Rgui,
            "xmb" => MenuDriver::Xmb,
            _ => MenuDriver::MaterialUi,
        }
    }

    pub fn spec(self) -> MenuDriverSpec {
        let index = Self::ALL
            .iter()
            .position(|driver| *driver == self)
            .unwrap_or(0);
        native_spec(unsafe { rf_menu_driver_at(index as c_uint) })
    }

    pub fn next_ident(current: &str) -> &'static str {
        let c_current = std::ffi::CString::new(current).unwrap_or_default();
        native_str(
            unsafe { rf_menu_driver_next_ident(c_current.as_ptr()) },
            "materialui",
        )
    }
}

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
                driver: "materialui".to_string(),
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
                    "RetroArch menu, play screen, library, drivers and paths",
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
        self.push_quick_menu_with_settings(has_game, None);
    }

    pub fn push_quick_menu_with_settings(&mut self, has_game: bool, settings: Option<&Settings>) {
        let state_slot = Self::state_slot_label(settings);
        let auto_save = settings
            .and_then(|s| s.get("savestate_auto_save"))
            .map_or("false", String::as_str);
        let auto_load = settings
            .and_then(|s| s.get("savestate_auto_load"))
            .map_or("false", String::as_str);
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
            Self::action(
                "Close Content",
                "Unload the current game and return to the menu",
                ACTION_CLOSE_CONTENT,
            ),
            Self::action(
                "Take Screenshot",
                "Write a screenshot to the screenshots directory",
                ACTION_TAKE_SCREENSHOT,
            ),
            Self::setting(
                "State Slot",
                "Select the active save-state slot (Auto, 0-999 like RetroArch)",
                &state_slot,
                ACTION_STATE_SLOT,
            ),
            Self::action(
                "State Slot -",
                "Move to the previous save-state slot; below 0 becomes Auto",
                ACTION_STATE_SLOT_DECREASE,
            ),
            Self::action(
                "State Slot +",
                "Move to the next save-state slot; above 999 wraps to 0",
                ACTION_STATE_SLOT_INCREASE,
            ),
            Self::action(
                "Save State",
                "Instantly serialize the current gameplay state",
                ACTION_SAVE_STATE_SLOT_0,
            ),
            Self::action(
                "Load State",
                "Restore the selected save-state slot",
                ACTION_LOAD_STATE_SLOT_0,
            ),
            Self::action(
                "Undo Load State",
                "Restore the state that existed before the last load",
                ACTION_UNDO_LOAD_STATE,
            ),
            Self::action(
                "Undo Save State",
                "Restore the overwritten save-state backup",
                ACTION_UNDO_SAVE_STATE,
            ),
            Self::action(
                "Save SRAM",
                "Flush battery-backed memory card / SRAM data",
                ACTION_SAVE_SRAM,
            ),
            Self::setting(
                "Auto Save State",
                "Save the active slot when content closes",
                auto_save,
                725,
            ),
            Self::setting(
                "Auto Load State",
                "Load the active slot after launch when present",
                auto_load,
                726,
            ),
            Self::submenu(
                "Core Options",
                "Variables exposed by the active libretro core",
                ACTION_CORE_OPTIONS,
            ),
            Self::submenu(
                "Controls",
                "Port controls, remaps, overlays and connected pads",
                ACTION_CONTROLS,
            ),
            Self::submenu(
                "Cheats",
                "Load, append, toggle and apply cheat files",
                ACTION_CHEATS,
            ),
            Self::submenu("Shaders", "Video shader passes and presets", ACTION_SHADERS),
            Self::submenu(
                "Overrides",
                "Core/content/game override configuration",
                ACTION_OVERRIDES,
            ),
            Self::submenu(
                "Disc Control",
                "Swap/eject virtual discs when a core supports it",
                ACTION_DISC_CONTROL,
            ),
            Self::submenu(
                "Display & Orientation",
                "Scaling, filtering and orientation controls",
                ACTION_DISPLAY_SETTINGS,
            ),
            Self::submenu(
                "Audio",
                "Mixer, mute and latency shortcuts",
                ACTION_AUDIO_MIXER,
            ),
            Self::submenu(
                "Replay",
                "Replay capture and playback controls",
                ACTION_REPLAY,
            ),
            Self::submenu(
                "Recording",
                "Recording output and driver settings",
                ACTION_RECORDING,
            ),
            Self::submenu(
                "Streaming",
                "Streaming output and service settings",
                ACTION_STREAMING,
            ),
            Self::action(
                "Add to Favorites",
                "Add current ROM to Favorites",
                ACTION_ADD_TO_FAVORITES,
            ),
            Self::action(
                "Add to Playlist",
                "Append current content to a playlist",
                ACTION_ADD_TO_PLAYLIST,
            ),
            Self::action(
                "Set Core Association",
                "Remember this core for the content extension",
                ACTION_SET_CORE_ASSOCIATION,
            ),
            Self::submenu(
                "Information",
                "Core and content runtime details",
                ACTION_CORE_INFORMATION,
            ),
        ];
        if has_game {
            entries.push(Self::action(
                "Exit Game",
                "Save SRAM, unload the current game and return to the library",
                ACTION_EXIT_GAME,
            ));
        }
        self.history.push(MenuList {
            title: "Quick Menu".to_string(),
            entries,
        });
    }

    fn state_slot_label(settings: Option<&Settings>) -> String {
        match settings
            .and_then(|settings| settings.get("state_slot"))
            .map(String::as_str)
        {
            Some("-1") => "Auto".to_string(),
            Some(value) if !value.is_empty() => value.to_string(),
            _ => "0".to_string(),
        }
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
                GfxBackendKind::Wgpu => "wgpu",
                GfxBackendKind::Metal => "wgpu-metal",
                GfxBackendKind::OpenGl => "wgpu-opengl",
                GfxBackendKind::Vulkan => "wgpu-vulkan",
                GfxBackendKind::MoltenVk => "wgpu-moltenvk",
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
        let mut bridge = NativeMenuBridge::from_settings(settings);
        let snapshot = bridge.sync_runtime_config();
        self.skin = MenuSkin {
            driver: snapshot.driver,
            theme: snapshot.theme,
            assets_directory: snapshot.assets_directory,
        };
    }

    pub fn push_skin_settings(&mut self, settings: &Settings) {
        self.apply_skin_from_settings(settings);
        let spec = MenuDriver::from_ident(&self.skin.driver).spec();
        let mut entries = vec![
            Self::setting(
                "Menu Engine",
                "Detached RetroArch menu drivers: materialui, ozone, xmb, rgui",
                format!("{} ({})", spec.display_name, spec.ident),
                ACTION_SKIN_SETTINGS,
            ),
            Self::setting(
                "Layout Model",
                "RetroArch driver layout translated into platform-neutral rows",
                spec.layout_model,
                ACTION_SKIN_SETTINGS + 1,
            ),
            Self::setting(
                "Input Model",
                "Pointer, touch, keyboard and RetroPad behavior for this driver",
                spec.input_model,
                ACTION_SKIN_SETTINGS + 2,
            ),
            Self::setting(
                "Thumbnails",
                "Driver-specific thumbnail placement and list treatment",
                spec.thumbnail_model,
                ACTION_SKIN_SETTINGS + 3,
            ),
            Self::setting(
                "Animation",
                "Driver-specific transition and selection behavior",
                spec.animation_model,
                ACTION_SKIN_SETTINGS + 4,
            ),
            Self::setting(
                "Theme",
                "Theme / color preset shared by all menu drivers",
                &self.skin.theme,
                ACTION_SKIN_SETTINGS + 5,
            ),
            Self::setting(
                "Skin Assets",
                "Common menu asset root used by the active RetroArch menu driver",
                &self.skin.assets_directory,
                ACTION_SKIN_SETTINGS + 6,
            ),
        ];
        entries.extend(self.driver_specific_skin_entries(settings, spec.ident));
        self.history.push(MenuList {
            title: "User Interface".to_string(),
            entries,
        });
    }

    fn driver_specific_skin_entries(&self, settings: &Settings, driver: &str) -> Vec<MenuEntry> {
        match driver {
            "ozone" => vec![
                Self::setting(
                    "Ozone Sidebar",
                    "Show/collapse the left category sidebar",
                    settings
                        .get("ozone_show_sidebar")
                        .map_or("true", String::as_str),
                    270,
                ),
                Self::setting(
                    "Ozone Header",
                    "Header icon and separator treatment",
                    settings
                        .get("ozone_header_style")
                        .map_or("icon_separator", String::as_str),
                    271,
                ),
                Self::setting(
                    "Ozone Padding",
                    "Density multiplier used by the shared renderer",
                    settings
                        .get("ozone_padding_factor")
                        .map_or("1.0", String::as_str),
                    272,
                ),
                Self::setting(
                    "Ozone Font Scale",
                    "Global and per-list text scale",
                    settings
                        .get("ozone_font_scale")
                        .map_or("1.0", String::as_str),
                    273,
                ),
                Self::setting(
                    "Ozone Thumbnail Scale",
                    "Metadata panel thumbnail scale",
                    settings
                        .get("ozone_thumbnail_scale_factor")
                        .map_or("1.0", String::as_str),
                    274,
                ),
            ],
            "materialui" => vec![
                Self::setting(
                    "Material Icons",
                    "Enable Material icon glyphs in list rows",
                    settings
                        .get("materialui_icons_enable")
                        .map_or("true", String::as_str),
                    270,
                ),
                Self::setting(
                    "Switch Icons",
                    "Use switch-style toggle indicators",
                    settings
                        .get("materialui_switch_icons")
                        .map_or("true", String::as_str),
                    271,
                ),
                Self::setting(
                    "Navigation Bar",
                    "Show the Material navigation bar",
                    settings
                        .get("materialui_show_nav_bar")
                        .map_or("true", String::as_str),
                    272,
                ),
                Self::setting(
                    "Auto Rotate Nav",
                    "Rotate navigation controls with orientation",
                    settings
                        .get("materialui_auto_rotate_nav_bar")
                        .map_or("true", String::as_str),
                    273,
                ),
                Self::setting(
                    "Dual Thumbnail List",
                    "Use dual-thumbnail list layout when space allows",
                    settings
                        .get("materialui_dual_thumbnail_list_view_enable")
                        .map_or("true", String::as_str),
                    274,
                ),
            ],
            "rgui" => vec![
                Self::setting(
                    "RGUI Theme",
                    "Classic terminal palette / preset",
                    settings
                        .get("rgui_menu_theme_preset")
                        .map_or("default", String::as_str),
                    270,
                ),
                Self::setting(
                    "RGUI Aspect",
                    "Aspect-ratio lock used by grid layout",
                    settings
                        .get("rgui_aspect_ratio")
                        .map_or("auto", String::as_str),
                    271,
                ),
                Self::setting(
                    "Inline Thumbnails",
                    "Draw thumbnails inside rows",
                    settings
                        .get("rgui_inline_thumbnails")
                        .map_or("false", String::as_str),
                    272,
                ),
                Self::setting(
                    "Extended ASCII",
                    "Use extended box drawing glyphs",
                    settings
                        .get("rgui_extended_ascii")
                        .map_or("true", String::as_str),
                    273,
                ),
                Self::setting(
                    "Full Width",
                    "Stretch list to the full viewport width",
                    settings
                        .get("rgui_full_width_layout")
                        .map_or("true", String::as_str),
                    274,
                ),
            ],
            "xmb" => vec![
                Self::setting(
                    "XMB Icons",
                    "Icon theme used for horizontal categories",
                    settings
                        .get("xmb_theme")
                        .map_or("monochrome", String::as_str),
                    270,
                ),
                Self::setting(
                    "Horizontal List",
                    "Show category carousel",
                    settings
                        .get("xmb_show_horizontal_list")
                        .map_or("true", String::as_str),
                    271,
                ),
                Self::setting(
                    "Shadows",
                    "Draw icon and text shadows",
                    settings
                        .get("xmb_shadows_enable")
                        .map_or("true", String::as_str),
                    272,
                ),
                Self::setting(
                    "Alpha Factor",
                    "Background alpha / wallpaper blending",
                    settings
                        .get("xmb_alpha_factor")
                        .map_or("75", String::as_str),
                    273,
                ),
                Self::setting(
                    "Layout",
                    "XMB column/row spacing preset",
                    settings.get("xmb_layout").map_or("auto", String::as_str),
                    274,
                ),
            ],
            _ => vec![],
        }
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
                    "Menu UI & Skin",
                    "Ozone, Material UI, RGUI, XMB, themes and assets",
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
                    settings
                        .get("menu_driver")
                        .map_or("materialui", String::as_str),
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
                    "Full-screen RetroArch menu overlay",
                    settings
                        .get("quick_menu_style")
                        .map_or("retroarch_fullscreen", String::as_str),
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

    pub fn push_save_state_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Save States".to_string(),
            entries: vec![
                Self::action(
                    "Save State",
                    "Instantly serialize the current gameplay state",
                    ACTION_SAVE_STATE_SLOT_0,
                ),
                Self::action(
                    "Load State",
                    "Restore the previously saved instant state",
                    ACTION_LOAD_STATE_SLOT_0,
                ),
                Self::action(
                    "Save SRAM Now",
                    "Flush battery-backed memory card / SRAM data",
                    ACTION_SAVE_SRAM,
                ),
                Self::setting(
                    "State Slot",
                    "Auto plus slots 0-999 match RetroArch hotkey bounds",
                    Self::state_slot_label(Some(settings)),
                    ACTION_STATE_SLOT,
                ),
                Self::action(
                    "State Slot -",
                    "Previous slot; below 0 becomes Auto",
                    ACTION_STATE_SLOT_DECREASE,
                ),
                Self::action(
                    "State Slot +",
                    "Next slot; above 999 wraps to 0",
                    ACTION_STATE_SLOT_INCREASE,
                ),
                Self::setting(
                    "Auto Save State",
                    "Save active slot when closing content",
                    settings
                        .get("savestate_auto_save")
                        .map_or("false", String::as_str),
                    725,
                ),
                Self::setting(
                    "Auto Load State",
                    "Load active slot after launching content when present",
                    settings
                        .get("savestate_auto_load")
                        .map_or("false", String::as_str),
                    726,
                ),
                Self::setting(
                    "Savefile Directory",
                    "SRAM and memory card saves",
                    settings.savefile_directory().to_string_lossy(),
                    722,
                ),
                Self::setting(
                    "Savestate Directory",
                    "Instant save state files",
                    settings.savestate_directory().to_string_lossy(),
                    723,
                ),
            ],
        });
    }

    pub fn push_shader_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Shaders".to_string(),
            entries: vec![
                Self::setting(
                    "Video Shader Directory",
                    "RetroArch shader presets",
                    settings
                        .path_value("video_shader_dir")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    730,
                ),
                Self::setting(
                    "Video Filter Directory",
                    "CPU video filters",
                    settings
                        .path_value("video_filter_dir")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    731,
                ),
                Self::action(
                    "Load Preset",
                    "Open a shader preset from the shader directory",
                    732,
                ),
                Self::action(
                    "Save Preset",
                    "Save current shader parameters as a preset",
                    733,
                ),
                Self::action("Remove Shader Passes", "Clear active shader passes", 734),
            ],
        });
    }

    pub fn push_cheat_settings(&mut self, settings: &Settings) {
        self.history.push(MenuList {
            title: "Cheats".to_string(),
            entries: vec![
                Self::setting(
                    "Cheat File Directory",
                    "RetroArch cheat database path",
                    settings
                        .path_value("cheat_database_path")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    740,
                ),
                Self::action("Load Cheat File", "Replace active cheats from a file", 741),
                Self::action("Append Cheat File", "Append cheats from another file", 742),
                Self::action(
                    "Apply Changes",
                    "Apply edited cheat toggles to the running core",
                    743,
                ),
            ],
        });
    }

    pub fn push_override_settings(&mut self) {
        self.history.push(MenuList {
            title: "Overrides".to_string(),
            entries: vec![
                Self::action(
                    "Save Core Overrides",
                    "Save settings for the active core",
                    750,
                ),
                Self::action(
                    "Save Content Directory Overrides",
                    "Save settings for this content folder",
                    751,
                ),
                Self::action(
                    "Save Game Overrides",
                    "Save settings for the active game",
                    752,
                ),
                Self::action("Remove Overrides", "Delete active override files", 753),
            ],
        });
    }

    pub fn push_disc_control(&mut self) {
        self.history.push(MenuList {
            title: "Disc Control".to_string(),
            entries: vec![
                Self::action("Eject Disc", "Toggle the virtual tray state", 760),
                Self::setting("Current Disc Index", "Selected disc number", "1", 761),
                Self::action("Load New Disc", "Append or replace a disc image", 762),
                Self::action("Cycle Tray Status", "Close/eject the virtual tray", 763),
            ],
        });
    }

    pub fn push_replay_recording_settings(&mut self, title: &str, settings: &Settings) {
        self.history.push(MenuList {
            title: title.to_string(),
            entries: vec![
                Self::setting(
                    "Output Directory",
                    "RetroArch recording output path",
                    settings
                        .path_value("recording_output_directory")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    770,
                ),
                Self::setting(
                    "Config Directory",
                    "Recording profile directory",
                    settings
                        .path_value("recording_config_directory")
                        .unwrap_or_default()
                        .to_string_lossy(),
                    771,
                ),
                Self::action("Start", "Start this runtime capture mode", 772),
                Self::action("Stop", "Stop this runtime capture mode", 773),
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
            "Menu UI & Skin",
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
            "Save State",
            "Close Content",
            "Disc Control",
            "State Slot",
            "Auto Save State",
            "Auto Load State",
        ] {
            assert!(labels.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn native_menu_bridge_reads_rust_settings_and_assets() {
        let root = std::env::temp_dir().join(format!(
            "retrofront-menu-assets-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("ozone")).unwrap();
        let mut settings = Settings::new();
        settings.set("menu_driver", "ozone");
        settings.set("menu_theme", "dark");
        settings.set("menu_assets_directory", &root.to_string_lossy());
        std::fs::create_dir_all(root.join("ozone/png")).unwrap();
        std::fs::write(root.join("ozone/regular.ttf"), b"font").unwrap();
        std::fs::write(root.join("ozone/png/retroarch.png"), b"png").unwrap();
        let mut bridge = NativeMenuBridge::from_settings(&settings);
        let snapshot = bridge.sync_runtime_config();
        assert_eq!(snapshot.driver, "ozone");
        assert_eq!(snapshot.assets_directory, root.to_string_lossy());
        assert!(snapshot.assets_ready);
        let resolved = bridge.resolve_assets("ozone");
        assert!(resolved.assets_ready);
        assert!(resolved.icon_directory.ends_with("ozone/png"));
        assert!(resolved.font_path.ends_with("ozone/regular.ttf"));
        assert!(resolved
            .background_path
            .ends_with("ozone/png/retroarch.png"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn native_menu_library_reports_all_detached_sources() {
        let files = native_menu_source_files();
        assert_eq!(files.len(), 34);
        assert!(files.iter().all(|(_, compiled)| *compiled));
        assert!(files
            .iter()
            .any(|(path, _)| path.ends_with("drivers/ozone.c")));
        assert!(files
            .iter()
            .any(|(path, _)| path.ends_with("drivers/xmb.c")));
        assert!(files
            .iter()
            .any(|(path, _)| path.ends_with("menu_driver.c")));
    }

    #[test]
    fn native_menu_layouts_fit_common_viewports() {
        for driver in ["materialui", "ozone", "xmb", "rgui"] {
            let metrics = native_menu_layout(driver, 390, 844).expect("layout");
            assert_eq!(metrics.viewport_width, 390);
            assert_eq!(metrics.viewport_height, 844);
            assert!(metrics.content_x < metrics.viewport_width, "{driver}");
            assert!(metrics.content_y < metrics.viewport_height, "{driver}");
            assert!(metrics.content_width > 0, "{driver}");
            assert!(metrics.content_height > 0, "{driver}");
            assert!(metrics.row_height >= 22, "{driver}");
        }
    }

    #[test]
    fn menu_skin_settings_expose_retroarch_driver_models() {
        let mut engine = MenuEngine::new();
        let mut settings = Settings::new();
        for (driver, expected) in [
            ("ozone", "Ozone Sidebar"),
            ("materialui", "Material Icons"),
            ("rgui", "RGUI Theme"),
            ("xmb", "XMB Icons"),
        ] {
            settings.set("menu_driver", driver);
            engine.push_skin_settings(&settings);
            let labels: Vec<&str> = engine
                .current()
                .unwrap()
                .entries
                .iter()
                .map(|entry| entry.label.as_str())
                .collect();
            assert!(
                labels.contains(&"Menu Engine"),
                "missing driver selector for {driver}"
            );
            assert!(
                labels.contains(&expected),
                "missing {expected} for {driver}"
            );
            assert!(
                labels.contains(&"Layout Model"),
                "missing layout model for {driver}"
            );
        }
    }
}
