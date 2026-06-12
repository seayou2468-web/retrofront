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

use fs::HostFilesystem;
use input::InputSystem;
use menu::MenuModel;
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

        Self {
            menu: Arc::new(RwLock::new(MenuModel::default())),
            renderer: Arc::new(RwLock::new(VideoRenderer::new())),
            input: Arc::new(RwLock::new(InputSystem::new())),
            filesystem,
            settings,
            tasks,
            playlists,
            shaders,
        }
    }

    /// Advance non-render menu services once per frame.
    pub fn tick(&self) {
        self.tasks.poll_completed();
        self.input.write().begin_frame();
    }
}
