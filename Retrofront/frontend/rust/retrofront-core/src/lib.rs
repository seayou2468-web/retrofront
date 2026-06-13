//! Rust backend for the fixed `Retrofront/frontend/menu` C menu contract.
//!
//! The C menu remains the specification owner.  This crate supplies the systems
//! that menu code expects (video, input, filesystem, settings, tasks, playlists,
//! shader management and libretro core loading) through Rust traits plus a small
//! C ABI shim.  Platform code for Linux and physical iOS devices should host one
//! [`RetrofrontRuntime`] instance and expose it to the existing menu callbacks.

pub mod c_api;
pub mod core;
pub mod fs;
pub mod input;
pub mod libretro;
pub mod menu;
pub mod playlist;
pub mod renderer;
pub mod settings;
pub mod shader;
pub mod task;

use std::sync::Arc;
use std::{fs as std_fs, path::PathBuf};

use fs::HostFilesystem;
use input::InputSystem;
use menu::{MenuDriver, MenuEntry, MenuEntryType, MenuIntent, MenuModel};
use parking_lot::RwLock;
use playlist::PlaylistStore;
use renderer::VideoRenderer;
use settings::SettingsStore;
use shader::ShaderManager;
use task::TaskSystem;

/// Shared runtime services consumed by the `menu/` implementation.
#[derive(Clone)]
pub struct RetrofrontRuntime {
    pub menu: Arc<RwLock<MenuModel>>,
    pub renderer: Arc<RwLock<VideoRenderer>>,
    pub input: Arc<RwLock<InputSystem>>,
    pub filesystem: HostFilesystem,
    pub settings: SettingsStore,
    pub tasks: TaskSystem,
    pub playlists: PlaylistStore,
    pub shaders: Arc<RwLock<ShaderManager>>,
}

impl RetrofrontRuntime {
    /// Construct a platform-neutral runtime rooted at `data_dir`.
    pub fn new(data_dir: impl Into<std::path::PathBuf>) -> Self {
        let filesystem = HostFilesystem::new(data_dir.into());
        let settings = SettingsStore::new(filesystem.config_dir());
        let tasks = TaskSystem::new();
        let playlists = PlaylistStore::new(filesystem.playlists_dir());
        let shaders = Arc::new(RwLock::new(ShaderManager::new(filesystem.shader_dir())));

        let runtime = Self {
            menu: Arc::new(RwLock::new(MenuModel::default())),
            renderer: Arc::new(RwLock::new(VideoRenderer::new())),
            input: Arc::new(RwLock::new(InputSystem::new())),
            filesystem,
            settings,
            tasks,
            playlists,
            shaders,
        };
        runtime.install_default_bindings();
        runtime.rebuild_home_menu();
        runtime
    }

    /// Install menu defaults used by C menu drivers and platform shells.
    pub fn install_default_bindings(&self) {
        let mut input = self.input.write();
        // Common desktop keys. iOS touch/gamepad code can add platform-specific
        // bindings without changing menu code.
        input.bind(input::InputSource::Key(38), input::MenuAction::Up);
        input.bind(input::InputSource::Key(40), input::MenuAction::Down);
        input.bind(input::InputSource::Key(37), input::MenuAction::Left);
        input.bind(input::InputSource::Key(39), input::MenuAction::Right);
        input.bind(input::InputSource::Key(13), input::MenuAction::Ok);
        input.bind(input::InputSource::Key(27), input::MenuAction::Cancel);
    }

    pub fn prepare_storage(&self) -> std::io::Result<()> {
        self.filesystem.ensure_layout()?;
        self.settings.load()?;
        if let Some(settings::SettingValue::String(name)) = self.settings.get("menu_driver") {
            if let Some(driver) = MenuDriver::from_name(&name) {
                self.menu.write().set_driver(driver);
            }
        }
        self.load_menu_assets();
        Ok(())
    }

    pub fn load_menu_assets(&self) -> usize {
        let mut renderer = self.renderer.write();
        let mut loaded = 0;
        for root in [
            self.filesystem.assets_dir(),
            self.filesystem.assets_dir().join("overlays"),
            self.filesystem.overlays_dir(),
            self.filesystem.fonts_dir(),
        ] {
            loaded += renderer.load_menu_assets_from(root);
        }
        for driver in menu::FIXED_MENU_DRIVERS {
            let descriptor = driver.descriptor();
            for rel in [descriptor.asset_dir, descriptor.font_dir] {
                if !rel.is_empty() {
                    loaded += renderer.load_menu_assets_from(self.filesystem.root().join(rel));
                }
            }
        }
        loaded
    }

    /// Advance non-render menu services once per frame.
    pub fn tick(&self) {
        self.tasks.poll_completed();
        self.input.write().begin_frame();
    }

    /// Rebuild the root menu from Rust-owned services so the fixed C menu has
    /// concrete, navigable entries immediately after startup.
    pub fn rebuild_home_menu(&self) {
        let entries = vec![
            MenuEntry {
                label: "Load Content".into(),
                sublabel: "Open the UI-only content browser mock".into(),
                path: "retrofront://content".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Playlists".into(),
                sublabel: "Browse mock playlists and entries".into(),
                path: "retrofront://playlists".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Cores".into(),
                sublabel: "Select a mock libretro core".into(),
                path: "retrofront://cores".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Shaders".into(),
                sublabel: "Preview librashader preset UI without applying it".into(),
                path: "retrofront://shaders".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Settings".into(),
                sublabel: "Open UI-only settings pages".into(),
                path: "retrofront://settings".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Online Updater".into(),
                sublabel: "Show mock progress and disabled updater actions".into(),
                path: "retrofront://online-updater".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Information".into(),
                sublabel: "Runtime, renderer and driver status".into(),
                path: "retrofront://information".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Quit Retrofront".into(),
                sublabel: "Open the exit confirmation UI only".into(),
                path: "retrofront://quit".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
        ];
        self.menu.write().set_root("Retrofront", entries);
    }

    /// Execute high-level menu intents produced by [`MenuModel`].
    pub fn dispatch_menu_intent(&self, intent: MenuIntent) {
        match intent {
            MenuIntent::OpenPath(path) => self.open_menu_path(path),
            MenuIntent::LaunchContent {
                core_path,
                game_path,
            } if !core_path.is_empty() && !game_path.is_empty() => {
                self.settings.set(
                    "pending_core_path",
                    settings::SettingValue::String(core_path),
                );
                self.settings.set(
                    "pending_game_path",
                    settings::SettingValue::String(game_path),
                );
                let _ = self.settings.save();
            }
            MenuIntent::ToggleBool(_) | MenuIntent::Back | MenuIntent::LaunchContent { .. } => {}
        }
    }

    fn open_menu_path(&self, path: String) {
        let (title, entries): (String, Vec<MenuEntry>) = match path.as_str() {
            "retrofront://content" => (
                "Load Content".into(),
                vec![
                    dir_entry(
                        "Start Directory",
                        "retrofront://directory-browser",
                        "Mock file browser with folders and supported content",
                    ),
                    action_entry(
                        "Metroid Fusion.gba",
                        "mock://content/metroid-fusion",
                        "Nintendo Game Boy Advance / no core launched",
                    ),
                    action_entry(
                        "Chrono Trigger.sfc",
                        "mock://content/chrono-trigger",
                        "Super Nintendo / no core launched",
                    ),
                    action_entry(
                        "Sonic The Hedgehog.md",
                        "mock://content/sonic",
                        "Mega Drive / unsupported extension mock",
                    ),
                ],
            ),
            "retrofront://directory-browser" => (
                "Start Directory".into(),
                vec![
                    dir_entry(
                        "Nintendo - Game Boy Advance",
                        "retrofront://mock-folder/gba",
                        "3 mock files",
                    ),
                    dir_entry(
                        "Nintendo - Super Nintendo",
                        "retrofront://mock-folder/snes",
                        "2 mock files",
                    ),
                    action_entry(
                        "README.txt",
                        "mock://disabled/text-file",
                        "Disabled: not launchable content",
                    ),
                ],
            ),
            "retrofront://mock-folder/gba" => (
                "Nintendo - Game Boy Advance".into(),
                vec![
                    action_entry(
                        "Advance Wars.gba",
                        "mock://content/advance-wars",
                        "Would open core selection",
                    ),
                    action_entry(
                        "Castlevania - Aria of Sorrow.gba",
                        "mock://content/aria",
                        "Would open core selection",
                    ),
                    action_entry(
                        "The Legend of Zelda - Minish Cap.gba",
                        "mock://content/minish-cap",
                        "Long label clipping test",
                    ),
                ],
            ),
            "retrofront://mock-folder/snes" => (
                "Nintendo - Super Nintendo".into(),
                vec![
                    action_entry(
                        "Super Metroid.sfc",
                        "mock://content/super-metroid",
                        "Would open core selection",
                    ),
                    action_entry(
                        "ファイナルファンタジー VI.sfc",
                        "mock://content/ff6-jp",
                        "Japanese label rendering test",
                    ),
                ],
            ),
            "retrofront://playlists" => (
                "Playlists".into(),
                vec![
                    dir_entry(
                        "Favorites",
                        "playlist://Favorites",
                        "Pinned mock content with thumbnails",
                    ),
                    dir_entry(
                        "History",
                        "playlist://History",
                        "Recently opened mock content",
                    ),
                    dir_entry(
                        "Nintendo - Game Boy Advance",
                        "playlist://Nintendo - Game Boy Advance",
                        "Large list and long label checks",
                    ),
                    dir_entry(
                        "Empty Playlist",
                        "playlist://Empty Playlist",
                        "Empty-state UI check",
                    ),
                ],
            ),
            "retrofront://cores" => (
                "Cores".into(),
                vec![
                    enum_entry(
                        "mGBA",
                        "core://mgba",
                        "Nintendo Game Boy Advance / mock selected core",
                        false,
                    ),
                    enum_entry(
                        "Snes9x",
                        "core://snes9x",
                        "Super Nintendo / mock selected core",
                        false,
                    ),
                    enum_entry(
                        "SameBoy",
                        "core://sameboy",
                        "Game Boy / disabled placeholder",
                        false,
                    ),
                    action_entry(
                        "Download More Cores...",
                        "mock://online/core-download",
                        "Opens mock online updater progress",
                    ),
                ],
            ),
            "retrofront://shaders" => (
                "Shaders".into(),
                vec![
                    enum_entry(
                        "crt-royale.slangp",
                        "shader://crt-royale.slangp",
                        "librashader raw-handle path / not applied yet",
                        false,
                    ),
                    enum_entry(
                        "sharp-bilinear.slangp",
                        "shader://sharp-bilinear.slangp",
                        "Preview placeholder only",
                        false,
                    ),
                    enum_entry(
                        "lcd-grid-v2.slangp",
                        "shader://lcd-grid-v2.slangp",
                        "Parameter UI mock",
                        false,
                    ),
                    dir_entry(
                        "Shader Parameters",
                        "retrofront://shader-parameters",
                        "Mock sliders and toggles",
                    ),
                ],
            ),
            "retrofront://shader-parameters" => (
                "Shader Parameters".into(),
                vec![
                    value_entry(
                        "Mask Strength",
                        "0.70",
                        "Mock float value; left/right changes will be added later",
                    ),
                    value_entry("Scanline Weight", "0.35", "Mock float value"),
                    enum_entry(
                        "Integer Scale",
                        "mock://toggle/integer-scale",
                        "Enabled UI-only toggle",
                        true,
                    ),
                ],
            ),
            "retrofront://settings" => (
                "Settings".into(),
                vec![
                    MenuEntry {
                        label: "Menu driver".into(),
                        value: self.menu.read().driver().as_name().into(),
                        sublabel: "Uses fixed C menu drivers: ozone/xmb/materialui/rgui".into(),
                        path: "retrofront://menu-drivers".into(),
                        entry_type: MenuEntryType::Enum,
                        ..Default::default()
                    },
                    MenuEntry {
                        label: "Video".into(),
                        sublabel: "wgpu renderer, scaling and fullscreen mock".into(),
                        path: "retrofront://settings/video".into(),
                        entry_type: MenuEntryType::Dir,
                        ..Default::default()
                    },
                    MenuEntry {
                        label: "Input".into(),
                        sublabel: "Keyboard, gamepad, mouse and touch mappings".into(),
                        path: "retrofront://settings/input".into(),
                        entry_type: MenuEntryType::Dir,
                        ..Default::default()
                    },
                    MenuEntry {
                        label: "User Interface".into(),
                        sublabel: "Language, thumbnails, animations and menu visibility".into(),
                        path: "retrofront://settings/ui".into(),
                        entry_type: MenuEntryType::Dir,
                        ..Default::default()
                    },
                ],
            ),
            "retrofront://settings/video" => (
                "Video".into(),
                vec![
                    value_entry(
                        "Video driver",
                        "wgpu",
                        "Renderer backend fixed for the UI milestone",
                    ),
                    value_entry(
                        "Shader runtime",
                        "librashader raw handles",
                        "Not the librashader wgpu runtime",
                    ),
                    enum_entry("VSync", "mock://toggle/vsync", "Mock enabled toggle", true),
                    value_entry("Scale", "Windowed", "Resize and DPI UI path"),
                ],
            ),
            "retrofront://settings/input" => (
                "Input".into(),
                vec![
                    value_entry("Device 1", "Keyboard", "Arrow keys / Enter / Escape"),
                    value_entry("Device 2", "Gamepad", "Mock connected controller"),
                    enum_entry(
                        "Touch Gestures",
                        "mock://toggle/touch",
                        "Tap, scroll and fling path for materialui",
                        true,
                    ),
                ],
            ),
            "retrofront://settings/ui" => (
                "User Interface".into(),
                vec![
                    value_entry(
                        "Language",
                        "日本語 / English",
                        "Long and CJK label coverage",
                    ),
                    enum_entry(
                        "Thumbnails",
                        "mock://toggle/thumbnails",
                        "Placeholder image area",
                        true,
                    ),
                    enum_entry(
                        "Menu Animations",
                        "mock://toggle/animations",
                        "Alpha and scroll transitions",
                        true,
                    ),
                    value_entry(
                        "Theme",
                        self.menu.read().driver().as_name(),
                        "Current menu driver theme",
                    ),
                ],
            ),
            "retrofront://menu-drivers" => (
                "Menu Driver".into(),
                ["ozone", "xmb", "materialui", "rgui"]
                    .into_iter()
                    .map(|name| MenuEntry {
                        label: name.into(),
                        path: format!("menu-driver://{name}"),
                        checked: self.menu.read().driver().as_name() == name,
                        entry_type: MenuEntryType::Enum,
                        ..Default::default()
                    })
                    .collect(),
            ),
            _ if path.starts_with("menu-driver://") => {
                let name = path.trim_start_matches("menu-driver://");
                if let Some(driver) = MenuDriver::from_name(name) {
                    self.menu.write().set_driver(driver);
                    self.settings.set(
                        "menu_driver",
                        settings::SettingValue::String(driver.as_name().into()),
                    );
                    let _ = self.settings.save();
                }
                (
                    "Menu Driver".into(),
                    ["ozone", "xmb", "materialui", "rgui"]
                        .into_iter()
                        .map(|name| MenuEntry {
                            label: name.into(),
                            path: format!("menu-driver://{name}"),
                            checked: self.menu.read().driver().as_name() == name,
                            entry_type: MenuEntryType::Enum,
                            ..Default::default()
                        })
                        .collect(),
                )
            }
            _ if path.starts_with("playlist://") => {
                let name = path.trim_start_matches("playlist://");
                (name.into(), self.entries_for_mock_playlist(name))
            }
            _ if path.starts_with("shader://") => {
                let preset = path.trim_start_matches("shader://");
                (
                    "Shaders".into(),
                    vec![
                        enum_entry(
                            preset,
                            "",
                            "Selected in UI only; shader is not applied in this milestone",
                            true,
                        ),
                        value_entry("Preview", "placeholder", "Rendered by wgpu UI path"),
                        value_entry(
                            "Raw-handle runtime",
                            "available by design",
                            "librashader wgpu runtime remains unused",
                        ),
                    ],
                )
            }
            _ if path.starts_with("core://") => {
                let core = path.trim_start_matches("core://").to_owned();
                self.settings
                    .set("selected_core_path", settings::SettingValue::String(core));
                let _ = self.settings.save();
                (
                    "Cores".into(),
                    vec![
                        enum_entry(&core, "", "Selected core is stored in UI mock state", true),
                        action_entry(
                            "Load Content With This Core",
                            "retrofront://content",
                            "Return to content browser",
                        ),
                    ],
                )
            }
            _ if path.starts_with("content://") => {
                let game = path.trim_start_matches("content://").to_owned();
                self.settings
                    .set("pending_game_path", settings::SettingValue::String(game));
                if let Some(settings::SettingValue::String(core)) =
                    self.settings.get("selected_core_path")
                {
                    self.settings
                        .set("pending_core_path", settings::SettingValue::String(core));
                }
                let _ = self.settings.save();
                (
                    "Load Content".into(),
                    self.entries_for_rom_dir(self.filesystem.imports_dir()),
                )
            }
            "retrofront://online-updater" => (
                "Online Updater".into(),
                vec![
                    action_entry(
                        "Update Core Info Files",
                        "mock://progress/core-info",
                        "Shows mock 100% complete notification",
                    ),
                    action_entry(
                        "Update Assets",
                        "mock://progress/assets",
                        "Disabled until real network exists",
                    ),
                    action_entry(
                        "Update Thumbnails",
                        "mock://progress/thumbnails",
                        "Placeholder thumbnails stay visible",
                    ),
                ],
            ),
            "retrofront://information" => (
                "Information".into(),
                vec![
                    value_entry(
                        "UI milestone",
                        "50%",
                        "Display and page navigation complete path",
                    ),
                    value_entry("Renderer", "wgpu", "Command-list UI renderer"),
                    value_entry(
                        "Shader bridge",
                        "librashader raw handles",
                        "No librashader wgpu runtime",
                    ),
                    value_entry(
                        "Menu source",
                        "Retrofront/frontend/menu",
                        "Already tracking reference UI",
                    ),
                ],
            ),
            "retrofront://quit" => (
                "Quit Retrofront".into(),
                vec![
                    action_entry("Cancel", "mock://back", "Return with Back/Escape"),
                    action_entry(
                        "Quit UI Demo",
                        "mock://disabled/quit",
                        "Confirmation UI only; process is not closed",
                    ),
                ],
            ),
            _ if path.starts_with("mock://") => (
                "Not Implemented".into(),
                vec![
                    value_entry(
                        "Action",
                        path.as_str(),
                        "This UI route is intentionally mocked",
                    ),
                    action_entry(
                        "Back",
                        "mock://back",
                        "Use Cancel/Escape to return to the previous page",
                    ),
                ],
            ),
            _ => return,
        };
        self.menu.write().push_with_title(title, entries);
    }

    fn entries_for_mock_playlist(&self, name: &str) -> Vec<MenuEntry> {
        if name == "Empty Playlist" {
            return vec![value_entry(
                "No entries",
                "empty",
                "Empty-state UI is displayed without querying a real database",
            )];
        }
        (1..=12)
            .map(|i| {
                let label = match i {
                    1 => "Metroid Fusion".to_owned(),
                    2 => "Chrono Trigger".to_owned(),
                    3 => "ファイナルファンタジー VI".to_owned(),
                    4 => "The Legend of Zelda - The Minish Cap".to_owned(),
                    _ => format!("{name} Mock Entry {i:02}"),
                };
                MenuEntry {
                    label,
                    path: format!("mock://playlist/{name}/{i}"),
                    sublabel: "Thumbnail placeholder / launch disabled until UI is complete".into(),
                    entry_type: MenuEntryType::Action,
                    ..Default::default()
                }
            })
            .collect()
    }

    fn entries_for_playlists(&self) -> Vec<MenuEntry> {
        self.playlists
            .list()
            .unwrap_or_default()
            .into_iter()
            .map(|name| MenuEntry {
                label: name.clone(),
                path: format!("playlist://{name}"),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            })
            .collect()
    }

    fn entries_for_playlist(&self, name: &str) -> Vec<MenuEntry> {
        self.playlists
            .load(name)
            .map(|p| {
                p.entries
                    .into_iter()
                    .map(|e| MenuEntry {
                        label: e.label,
                        path: format!(
                            "launch://{}|{}",
                            e.core_path.unwrap_or_default().display(),
                            e.path.display()
                        ),
                        sublabel: e.core_name.unwrap_or_default(),
                        entry_type: MenuEntryType::Action,
                        ..Default::default()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn entries_for_rom_dir(&self, dir: PathBuf) -> Vec<MenuEntry> {
        self.entries_for_files(
            dir,
            &["gba", "gb", "gbc", "sfc", "smc", "nes", "zip"],
            "content://",
        )
    }

    fn entries_for_files(&self, dir: PathBuf, exts: &[&str], scheme: &str) -> Vec<MenuEntry> {
        let mut out = Vec::new();
        if let Ok(read_dir) = std_fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let ext_ok = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| exts.iter().any(|x| x.eq_ignore_ascii_case(e)))
                    .unwrap_or(false);
                if ext_ok {
                    out.push(MenuEntry {
                        label: path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Content")
                            .into(),
                        path: format!("{scheme}{}", path.display()),
                        entry_type: MenuEntryType::Action,
                        ..Default::default()
                    });
                }
            }
        }
        out.sort_by(|a, b| a.label.cmp(&b.label));
        out
    }
}

fn dir_entry(
    label: impl Into<String>,
    path: impl Into<String>,
    sublabel: impl Into<String>,
) -> MenuEntry {
    MenuEntry {
        label: label.into(),
        path: path.into(),
        sublabel: sublabel.into(),
        entry_type: MenuEntryType::Dir,
        ..Default::default()
    }
}

fn action_entry(
    label: impl Into<String>,
    path: impl Into<String>,
    sublabel: impl Into<String>,
) -> MenuEntry {
    MenuEntry {
        label: label.into(),
        path: path.into(),
        sublabel: sublabel.into(),
        entry_type: MenuEntryType::Action,
        ..Default::default()
    }
}

fn value_entry(
    label: impl Into<String>,
    value: impl Into<String>,
    sublabel: impl Into<String>,
) -> MenuEntry {
    MenuEntry {
        label: label.into(),
        value: value.into(),
        sublabel: sublabel.into(),
        entry_type: MenuEntryType::String,
        ..Default::default()
    }
}

fn enum_entry(
    label: impl Into<String>,
    path: impl Into<String>,
    sublabel: impl Into<String>,
    checked: bool,
) -> MenuEntry {
    MenuEntry {
        label: label.into(),
        path: path.into(),
        sublabel: sublabel.into(),
        checked,
        entry_type: MenuEntryType::Enum,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::MenuAction;

    #[test]
    fn ui_mock_pages_can_be_opened_without_real_backend_features() {
        let root =
            std::env::temp_dir().join(format!("retrofront-ui-runtime-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let runtime = RetrofrontRuntime::new(&root);
        runtime.rebuild_home_menu();

        assert_eq!(runtime.menu.read().current_entries().len(), 8);
        runtime.dispatch_menu_intent(MenuIntent::OpenPath("retrofront://playlists".into()));
        assert_eq!(runtime.menu.read().title(), "Playlists");
        assert!(runtime
            .menu
            .read()
            .current_entries()
            .iter()
            .any(|entry| entry.label == "Favorites"));

        runtime.dispatch_menu_intent(MenuIntent::OpenPath("playlist://Favorites".into()));
        assert_eq!(runtime.menu.read().title(), "Favorites");
        assert!(runtime.menu.read().current_entries().len() >= 12);

        runtime.dispatch_menu_intent(MenuIntent::OpenPath("retrofront://settings/video".into()));
        assert_eq!(runtime.menu.read().title(), "Video");
        assert!(runtime
            .menu
            .read()
            .current_entries()
            .iter()
            .any(|entry| entry.value == "wgpu"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ui_navigation_and_snapshot_cover_page_transitions() {
        let root =
            std::env::temp_dir().join(format!("retrofront-ui-snapshot-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let runtime = RetrofrontRuntime::new(&root);
        runtime.rebuild_home_menu();
        runtime.renderer.write().resize(800, 480);

        runtime.menu.write().action(MenuAction::Down);
        if let Some(intent) = runtime.menu.write().action(MenuAction::Ok) {
            runtime.dispatch_menu_intent(intent);
        }
        assert_eq!(runtime.menu.read().title(), "Playlists");

        let snapshot = root.join("playlists.ppm");
        runtime
            .renderer
            .read()
            .write_menu_snapshot_ppm(&runtime.menu.read(), &snapshot)
            .unwrap();
        let bytes = std::fs::read(&snapshot).unwrap();
        assert!(bytes.starts_with(b"P6\n800 480\n255\n"));
        assert!(bytes.len() > 800 * 480);
        let _ = std::fs::remove_dir_all(&root);
    }
}
