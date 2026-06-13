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
use menu::{MenuEntry, MenuEntryType, MenuIntent, MenuModel};
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
        Ok(())
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
                sublabel: "Browse imported ROMs".into(),
                path: "retrofront://content".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Playlists".into(),
                sublabel: "Open saved playlists".into(),
                path: "retrofront://playlists".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Cores".into(),
                sublabel: "Select a libretro core".into(),
                path: "retrofront://cores".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Shaders".into(),
                sublabel: "Load librashader presets".into(),
                path: "retrofront://shaders".into(),
                entry_type: MenuEntryType::Dir,
                ..Default::default()
            },
            MenuEntry {
                label: "Settings".into(),
                sublabel: "Frontend settings".into(),
                path: "retrofront://settings".into(),
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
                self.entries_for_rom_dir(self.filesystem.imports_dir()),
            ),
            "retrofront://playlists" => ("Playlists".into(), self.entries_for_playlists()),
            "retrofront://cores" => (
                "Cores".into(),
                self.entries_for_files(self.filesystem.cores_dir(), &["so", "dylib"], "core://"),
            ),
            "retrofront://shaders" => (
                "Shaders".into(),
                self.entries_for_files(
                    self.filesystem.shader_dir(),
                    &["slangp", "glslp", "cgp"],
                    "shader://",
                ),
            ),
            "retrofront://settings" => (
                "Settings".into(),
                vec![
                    MenuEntry {
                        label: "Video driver".into(),
                        value: "wgpu".into(),
                        sublabel: "Linux Vulkan / iOS Metal".into(),
                        ..Default::default()
                    },
                    MenuEntry {
                        label: "Shader runtime".into(),
                        value: "ordinary librashader from wgpu raw handles".into(),
                        ..Default::default()
                    },
                ],
            ),
            _ if path.starts_with("playlist://") => {
                let name = path.trim_start_matches("playlist://");
                (name.into(), self.entries_for_playlist(name))
            }
            _ if path.starts_with("shader://") => {
                let preset = path.trim_start_matches("shader://");
                let _ = self.shaders.write().set_preset(preset);
                (
                    "Shaders".into(),
                    self.entries_for_files(
                        self.filesystem.shader_dir(),
                        &["slangp", "glslp", "cgp"],
                        "shader://",
                    ),
                )
            }
            _ if path.starts_with("core://") => {
                let core = path.trim_start_matches("core://").to_owned();
                self.settings
                    .set("selected_core_path", settings::SettingValue::String(core));
                let _ = self.settings.save();
                (
                    "Cores".into(),
                    self.entries_for_files(
                        self.filesystem.cores_dir(),
                        &["so", "dylib"],
                        "core://",
                    ),
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
            _ => return,
        };
        self.menu.write().push_with_title(title, entries);
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
