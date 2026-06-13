use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    ffi::{c_char, CStr, CString},
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuDriver {
    Xmb,
    Ozone,
    Rgui,
    MaterialUi,
}
impl MenuDriver {
    pub fn all() -> [Self; 4] {
        [Self::Xmb, Self::Ozone, Self::Rgui, Self::MaterialUi]
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Xmb => "xmb",
            Self::Ozone => "ozone",
            Self::Rgui => "rgui",
            Self::MaterialUi => "materialui",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub disabled: bool,
    pub restart: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuItem {
    pub label: String,
    pub sublabel: String,
    pub value: String,
    pub enabled: bool,
    pub screen: Option<String>,
    pub thumbnail: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiRuntime {
    pub driver: MenuDriver,
    pub width: u32,
    pub height: u32,
    pub dpi_scale: f32,
    pub safe_area: [f32; 4],
    pub theme: String,
    pub language: String,
    pub selected: usize,
    pub stack: Vec<String>,
    pub settings: Vec<Setting>,
    pub notifications: VecDeque<String>,
    pub frame: u64,
    pub last_string: CString,
}

impl Default for UiRuntime {
    fn default() -> Self {
        Self::new(MenuDriver::Xmb)
    }
}
impl UiRuntime {
    pub fn new(driver: MenuDriver) -> Self {
        Self {
            driver,
            width: 1280,
            height: 720,
            dpi_scale: 1.0,
            safe_area: [0.0; 4],
            theme: "retro-dark".into(),
            language: "ja-JP".into(),
            selected: 0,
            stack: vec!["Main Menu".into()],
            settings: mock_settings(),
            notifications: VecDeque::from(["Retrofront UI runtime ready".into()]),
            frame: 0,
            last_string: CString::new("").unwrap(),
        }
    }
    pub fn begin_frame(&mut self, w: u32, h: u32, scale: f32) {
        self.width = w;
        self.height = h;
        self.dpi_scale = scale;
        self.frame += 1;
    }
    pub fn end_frame(&mut self) {
        while self.notifications.len() > 5 {
            self.notifications.pop_front();
        }
    }
    pub fn current_screen(&self) -> &str {
        self.stack.last().map(String::as_str).unwrap_or("Main Menu")
    }
    pub fn set_driver(&mut self, d: MenuDriver) {
        self.driver = d;
        self.selected = 0;
        self.notifications
            .push_back(format!("menu driver: {}", d.label()));
    }
    pub fn items(&self) -> Vec<MenuItem> {
        mock_items(self.current_screen())
    }
    pub fn activate(&mut self) {
        if let Some(item) = self.items().get(self.selected) {
            if !item.enabled {
                self.notifications
                    .push_back(format!("未実装: {}", item.label));
            } else if let Some(s) = &item.screen {
                self.stack.push(s.clone());
                self.selected = 0;
            } else {
                self.notifications
                    .push_back(format!("{}: mock success", item.label));
            }
        }
    }
    pub fn back(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
            self.selected = 0;
        } else {
            self.notifications
                .push_back("終了確認: UI mock keeps running".into());
        }
    }
    pub fn move_sel(&mut self, delta: isize) {
        let len = self.items().len().max(1);
        self.selected = ((self.selected as isize + delta).rem_euclid(len as isize)) as usize;
    }
    pub fn change_value(&mut self, dir: i32) {
        let len = self.settings.len();
        if len == 0 {
            return;
        }
        let idx = self.selected % len;
        if let Some(s) = self.settings.get_mut(idx) {
            if !s.disabled {
                s.value = format!("{} {}", s.value, if dir > 0 { "▶" } else { "◀" });
            }
        }
    }
    fn cstr(&mut self, s: String) -> *const c_char {
        self.last_string = CString::new(s).unwrap_or_default();
        self.last_string.as_ptr()
    }
}

pub fn mock_settings() -> Vec<Setting> {
    [
        "Video",
        "Audio",
        "Input",
        "User",
        "Directory",
        "Playlist",
        "Network",
        "Shader",
        "Menu",
    ]
    .iter()
    .enumerate()
    .map(|(i, c)| Setting {
        key: format!("{} / Mock Option", c),
        value: if i % 2 == 0 { "ON" } else { "Auto" }.into(),
        disabled: i == 6,
        restart: i == 0,
    })
    .collect()
}
fn item(l: &str, sub: &str, val: &str, en: bool, screen: Option<&str>, thumb: bool) -> MenuItem {
    MenuItem {
        label: l.into(),
        sublabel: sub.into(),
        value: val.into(),
        enabled: en,
        screen: screen.map(str::to_string),
        thumbnail: thumb,
    }
}
pub fn mock_items(screen: &str) -> Vec<MenuItem> {
    match screen {
        "Main Menu" => vec![
            item(
                "履歴",
                "Recently played content",
                "12",
                true,
                Some("History"),
                true,
            ),
            item(
                "プレイリスト",
                "Mock console playlists",
                "6",
                true,
                Some("Playlists"),
                true,
            ),
            item(
                "設定",
                "All RetroArch-style setting categories",
                "",
                true,
                Some("Settings"),
                false,
            ),
            item(
                "コアをロード",
                "Contentless and regular cores",
                "",
                true,
                Some("Cores"),
                false,
            ),
            item(
                "シェーダ",
                "Preset browser and preview",
                "",
                true,
                Some("Shaders"),
                false,
            ),
            item(
                "オンラインアップデータ",
                "Disabled mock actions",
                "",
                true,
                Some("Online Updater"),
                false,
            ),
            item(
                "情報",
                "System and build information",
                "",
                true,
                Some("Information"),
                false,
            ),
            item("終了", "Shows confirmation only", "", false, None, false),
        ],
        "Settings" => mock_settings()
            .into_iter()
            .map(|s| {
                item(
                    &s.key,
                    "In-memory UI setting",
                    &s.value,
                    !s.disabled,
                    None,
                    false,
                )
            })
            .collect(),
        "Playlists" => [
            "Nintendo - SNES",
            "Nintendo - Game Boy Advance",
            "Sega - Mega Drive",
            "Arcade",
            "Favorites",
            "日本語の長いプレイリスト名テスト",
        ]
        .iter()
        .map(|p| {
            item(
                p,
                "Playlist entries with thumbnails and metadata",
                "",
                true,
                Some("Playlist Entries"),
                true,
            )
        })
        .collect(),
        "Playlist Entries" | "History" => [
            "Chrono Trigger",
            "Super Metroid",
            "The Legend of Zelda: A Link to the Past",
            "Final Fantasy VI",
            "とても長い日本語タイトルのゲーム名サンプル",
        ]
        .iter()
        .map(|p| {
            item(
                p,
                "Core: mock libretro • Last played: today",
                "Ready",
                true,
                None,
                true,
            )
        })
        .collect(),
        "Cores" => [
            "Snes9x",
            "mGBA",
            "Genesis Plus GX",
            "MAME 2003-Plus",
            "2048 (contentless)",
        ]
        .iter()
        .map(|p| {
            item(
                p,
                "Core loading is mocked until UI is complete",
                "Installed",
                true,
                None,
                false,
            )
        })
        .collect(),
        "Shaders" => [
            "crt-royale.slangp",
            "lcd-grid.slangp",
            "nearest.slangp",
            "disabled-preview-warning",
        ]
        .iter()
        .enumerate()
        .map(|(i, p)| {
            item(
                p,
                "librashader raw-handle boundary placeholder",
                if i == 3 { "未実装" } else { "Preview" },
                i != 3,
                None,
                false,
            )
        })
        .collect(),
        "Online Updater" => [
            "Update Assets",
            "Update Core Info Files",
            "Update Databases",
            "Thumbnail Downloader",
        ]
        .iter()
        .map(|p| {
            item(
                p,
                "Network feature intentionally mocked",
                "未実装",
                false,
                None,
                false,
            )
        })
        .collect(),
        "Information" => vec![
            item(
                "Renderer",
                "wgpu command validation + egui/wgpu display",
                "OK",
                true,
                None,
                false,
            ),
            item(
                "Fonts",
                "Latin/Japanese fallback via egui font stack",
                "OK",
                true,
                None,
                false,
            ),
            item(
                "Input",
                "keyboard/mouse/touch/gamepad action mapping",
                "OK",
                true,
                None,
                false,
            ),
        ],
        _ => vec![item(
            "Empty",
            "No mock entries for this screen",
            "",
            false,
            None,
            false,
        )],
    }
}

fn parse_driver(p: *const c_char) -> MenuDriver {
    unsafe { CStr::from_ptr(p).to_string_lossy() }
        .as_ref()
        .parse()
        .unwrap_or(MenuDriver::Xmb)
}
impl std::str::FromStr for MenuDriver {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ozone" => Self::Ozone,
            "rgui" => Self::Rgui,
            "materialui" => Self::MaterialUi,
            _ => Self::Xmb,
        })
    }
}
#[no_mangle]
pub extern "C" fn retrofront_ui_runtime_create(driver: *const c_char) -> *mut UiRuntime {
    catch_unwind(|| Box::into_raw(Box::new(UiRuntime::new(parse_driver(driver)))))
        .unwrap_or(ptr::null_mut())
}
#[no_mangle]
pub extern "C" fn retrofront_ui_runtime_destroy(rt: *mut UiRuntime) {
    if !rt.is_null() {
        unsafe {
            drop(Box::from_raw(rt));
        }
    }
}
#[no_mangle]
pub extern "C" fn retrofront_ui_runtime_begin_frame(
    rt: *mut UiRuntime,
    w: u32,
    h: u32,
    scale: f32,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        rt.as_mut().map(|r| r.begin_frame(w, h, scale));
    }));
}
#[no_mangle]
pub extern "C" fn retrofront_ui_runtime_end_frame(rt: *mut UiRuntime) {
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        rt.as_mut().map(UiRuntime::end_frame);
    }));
}
#[no_mangle]
pub extern "C" fn retrofront_ui_runtime_get_screen(rt: *mut UiRuntime) -> *const c_char {
    catch_unwind(AssertUnwindSafe(|| unsafe {
        rt.as_mut()
            .map(|r| r.cstr(r.current_screen().to_string()))
            .unwrap_or(ptr::null())
    }))
    .unwrap_or(ptr::null())
}
