use retrofront_core::{
    FrontendCore, SessionState, UiCoreInfo, UiGameEntry, UiMenuList, UiSettingEntry,
};
use slint::{ComponentHandle, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel, Weak};
use std::cell::{Cell, RefCell};
use std::env;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

slint::include_modules!();

type SharedFrontend = Rc<RefCell<FrontendCore>>;

#[derive(Debug, Clone)]
struct StorageLayout {
    root: PathBuf,
    config_file: PathBuf,
    core_dir: PathBuf,
    bundled_core_dir: Option<PathBuf>,
    bundled_asset_dir: Option<PathBuf>,
    content_dir: PathBuf,
    assets_dir: PathBuf,
    info_dir: PathBuf,
    overlays_dir: PathBuf,
    downloads_dir: PathBuf,
    saves_dir: PathBuf,
    states_dir: PathBuf,
    system_dir: PathBuf,
    screenshots_dir: PathBuf,
    playlists_dir: PathBuf,
    cache_dir: PathBuf,
    overlay_config: PathBuf,
}

impl StorageLayout {
    fn current() -> Self {
        if cfg!(target_os = "ios") {
            Self::ios()
        } else {
            Self::linux()
        }
    }

    fn linux() -> Self {
        let home = env::var_os("HOME").map(PathBuf::from);
        let config_root = env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| home.as_ref().map(|home| home.join(".config")))
            .unwrap_or_else(|| PathBuf::from("retroarch"));
        let root = config_root.join("retroarch");
        Self::from_root(
            root,
            env_path("RETROFRONT_BUNDLED_CORE_DIR"),
            env_path("RETROFRONT_BUNDLED_ASSET_DIR"),
            false,
        )
    }

    fn ios() -> Self {
        let root = env_path("RETROFRONT_IOS_RETROARCH_ROOT")
            .or_else(|| env_path("HOME").map(|home| home.join("Documents").join("RetroArch")))
            .unwrap_or_else(|| PathBuf::from("RetroArch"));
        let exe_parent = env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf));
        let bundle_root = exe_parent
            .as_ref()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .or(exe_parent);
        let bundled_core_dir = env_path("RETROFRONT_BUNDLED_CORE_DIR")
            .or_else(|| bundle_root.as_ref().map(|root| root.join("Frameworks")));
        let bundled_asset_dir = env_path("RETROFRONT_BUNDLED_ASSET_DIR")
            .or_else(|| bundle_root.as_ref().map(|root| root.join("Resources")))
            .or(bundle_root);
        Self::from_root(root, bundled_core_dir, bundled_asset_dir, true)
    }

    fn from_root(
        root: PathBuf,
        bundled_core_dir: Option<PathBuf>,
        bundled_asset_dir: Option<PathBuf>,
        ios_layout: bool,
    ) -> Self {
        let config_dir = root.join("config");
        let core_dir = if ios_layout {
            bundled_core_dir
                .clone()
                .unwrap_or_else(|| root.join("cores"))
        } else {
            root.join("cores")
        };
        let info_dir = if ios_layout {
            root.join("info")
        } else {
            root.join("cores")
        };
        let content_dir = if ios_layout {
            root.join("Roms")
        } else {
            root.clone()
        };
        let cache_dir = if ios_layout {
            env::temp_dir()
        } else {
            root.join("temp")
        };
        let overlays_dir = root.join("overlays");
        Self {
            root: root.clone(),
            config_file: config_dir.join("retroarch.cfg"),
            core_dir,
            bundled_core_dir,
            bundled_asset_dir,
            content_dir,
            assets_dir: root.join("assets"),
            info_dir,
            overlays_dir: overlays_dir.clone(),
            downloads_dir: root.join("downloads"),
            saves_dir: root.join("saves"),
            states_dir: root.join("states"),
            system_dir: root.join("system"),
            screenshots_dir: root.join("screenshots"),
            playlists_dir: root.join("playlists"),
            cache_dir,
            overlay_config: overlays_dir.join("gamepads/flat/retropad.cfg"),
        }
    }

    fn create_directories(&self) -> Result<(), String> {
        for directory in [
            &self.root,
            self.config_file.parent().unwrap_or(&self.root),
            &self.assets_dir,
            &self.info_dir,
            &self.overlays_dir,
            &self.downloads_dir,
            &self.content_dir,
            &self.saves_dir,
            &self.states_dir,
            &self.system_dir,
            &self.screenshots_dir,
            &self.playlists_dir,
            &self.cache_dir,
        ] {
            std::fs::create_dir_all(directory)
                .map_err(|error| format!("create {}: {error}", directory.display()))?;
        }
        if !cfg!(target_os = "ios") {
            std::fs::create_dir_all(&self.core_dir)
                .map_err(|error| format!("create {}: {error}", self.core_dir.display()))?;
        }
        Ok(())
    }

    fn apply(&self, frontend: &mut FrontendCore) -> Result<(), String> {
        self.create_directories()?;
        frontend.set_base_dir(&self.root);
        frontend.set_setting("libretro_directory", path_string(&self.core_dir).as_str());
        frontend.set_setting("libretro_info_path", path_string(&self.info_dir).as_str());
        frontend.set_setting("assets_directory", path_string(&self.assets_dir).as_str());
        frontend.set_setting(
            "menu_assets_directory",
            path_string(&self.assets_dir).as_str(),
        );
        frontend.set_setting(
            "overlay_directory",
            path_string(&self.overlays_dir).as_str(),
        );
        frontend.set_setting("input_overlay", path_string(&self.overlay_config).as_str());
        frontend.set_setting("content_directory", path_string(&self.content_dir).as_str());
        frontend.set_setting("menu_content_directory", path_string(&self.root).as_str());
        frontend.set_setting(
            "core_assets_directory",
            path_string(&self.downloads_dir).as_str(),
        );
        frontend.set_setting("savefile_directory", path_string(&self.saves_dir).as_str());
        frontend.set_setting(
            "savestate_directory",
            path_string(&self.states_dir).as_str(),
        );
        frontend.set_setting("system_directory", path_string(&self.system_dir).as_str());
        frontend.set_setting(
            "screenshot_directory",
            path_string(&self.screenshots_dir).as_str(),
        );
        frontend.set_setting(
            "playlist_directory",
            path_string(&self.playlists_dir).as_str(),
        );
        frontend.set_setting("cache_directory", path_string(&self.cache_dir).as_str());
        frontend.set_info_dir(&self.info_dir);
        Ok(())
    }
}

pub fn run() -> Result<(), slint::PlatformError> {
    let layout = StorageLayout::current();
    let frontend = Rc::new(RefCell::new(FrontendCore::new()));
    let status = Rc::new(RefCell::new(String::from("Ready")));

    initialize_frontend(&frontend, &layout, &status);

    let window = MainWindow::new()?;
    refresh_window(&window, &frontend, &status);
    wire_callbacks(&window, frontend, layout, status);
    window.run()
}

#[no_mangle]
pub extern "C" fn retrofront_slint_ios_main() -> i32 {
    match run() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("Retrofront Slint UI failed: {error}");
            1
        }
    }
}

fn initialize_frontend(
    frontend: &SharedFrontend,
    layout: &StorageLayout,
    status: &Rc<RefCell<String>>,
) {
    let mut core = frontend.borrow_mut();
    if let Err(error) = layout.apply(&mut core) {
        *status.borrow_mut() = format!("Storage setup failed: {error}");
        return;
    }
    core.load_settings(&layout.config_file);
    let _ = layout.apply(&mut core);
    install_packaged_assets(&mut core, layout, status);
    if let Some(bundled) = &layout.bundled_core_dir {
        core.scan_cores(bundled);
    }
    core.scan_cores(&layout.core_dir);
    core.scan_configured_cores();
    core.scan_games(&layout.content_dir);
    core.save_settings();
}

fn install_packaged_assets(
    core: &mut FrontendCore,
    layout: &StorageLayout,
    status: &Rc<RefCell<String>>,
) {
    let Some(asset_dir) = &layout.bundled_asset_dir else {
        return;
    };
    for (name, destination) in [
        ("assets", &layout.assets_dir),
        ("info", &layout.info_dir),
        ("overlays", &layout.overlays_dir),
    ] {
        let zip = asset_dir.join(format!("{name}.zip"));
        if zip.exists() {
            match core.install_assets_zip(&zip, destination) {
                Ok(report) => {
                    *status.borrow_mut() = format!(
                        "Installed {name}: {} files, {} folders",
                        report.files_written, report.directories_created
                    );
                }
                Err(error) => *status.borrow_mut() = format!("Install {name} failed: {error}"),
            }
        }
    }
}

fn wire_callbacks(
    window: &MainWindow,
    frontend: SharedFrontend,
    layout: StorageLayout,
    status: Rc<RefCell<String>>,
) {
    let playing = Rc::new(Cell::new(false));

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_refresh_requested(move || {
            core.borrow_mut().scan_configured_cores();
            *status.borrow_mut() = "Refreshed cores, games, menu, and settings".into();
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, {
        let content_dir = layout.content_dir.clone();
        move |ui, core, status| {
            let ui_handle = ui.as_weak();
            ui.on_scan_games_requested(move || {
                core.borrow_mut().scan_games(&content_dir);
                *status.borrow_mut() = format!("Scanned {}", content_dir.display());
                refresh_weak(&ui_handle, &core, &status);
            });
        }
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_load_core_requested(move |index| {
            let selected = { core.borrow().core_summaries().get(index as usize).cloned() };
            match selected {
                Some(core_info) => match core.borrow_mut().load_core(&core_info.path) {
                    Ok(()) => {
                        *status.borrow_mut() = format!("Loaded core: {}", core_info.display_name)
                    }
                    Err(error) => *status.borrow_mut() = format!("Core load failed: {error}"),
                },
                None => *status.borrow_mut() = "Selected core is no longer available".into(),
            }
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_launch_game_requested(move |index| {
            let selected = { core.borrow().game_summaries().get(index as usize).cloned() };
            match selected {
                Some(game) => match core.borrow_mut().launch_content(&game.path, None, None) {
                    Ok(_) => *status.borrow_mut() = format!("Launched game: {}", game.label),
                    Err(error) => *status.borrow_mut() = format!("Launch failed: {error}"),
                },
                None => *status.borrow_mut() = "Selected game is no longer available".into(),
            }
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_frame_requested(move || {
            run_frame_once(&core, &status);
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, {
        let playing = playing.clone();
        move |ui, core, status| {
            let ui_handle = ui.as_weak();
            ui.on_play_pause_requested(move || {
                playing.set(!playing.get());
                *status.borrow_mut() = if playing.get() {
                    "Playback timer started"
                } else {
                    "Playback timer paused"
                }
                .into();
                refresh_weak(&ui_handle, &core, &status);
            });
        }
    });

    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(16), {
        let weak = window.as_weak();
        let core = frontend.clone();
        let status = status.clone();
        let playing = playing.clone();
        move || {
            if playing.get() {
                run_frame_once(&core, &status);
                if let Some(ui) = weak.upgrade() {
                    refresh_window(&ui, &core, &status);
                }
            }
        }
    });
    std::mem::forget(timer);

    on(window, &frontend, &status, {
        let playing = playing.clone();
        move |ui, core, status| {
            let ui_handle = ui.as_weak();
            ui.on_stop_requested(move || {
                playing.set(false);
                {
                    let mut borrowed = core.borrow_mut();
                    let _ = borrowed.save_save_ram();
                    borrowed.unload_game();
                }
                *status.borrow_mut() = "Stopped content and saved SRAM".into();
                refresh_weak(&ui_handle, &core, &status);
            });
        }
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_reset_requested(move || {
            match core.borrow_mut().reset() {
                Ok(()) => *status.borrow_mut() = "Content reset".into(),
                Err(error) => *status.borrow_mut() = format!("Reset failed: {error}"),
            }
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_save_state_requested(move || {
            match core.borrow_mut().save_state(0) {
                Ok(path) => *status.borrow_mut() = format!("Saved state: {}", path.display()),
                Err(error) => *status.borrow_mut() = format!("Save state failed: {error}"),
            }
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_load_state_requested(move || {
            match core.borrow_mut().load_state(0) {
                Ok(()) => *status.borrow_mut() = "Loaded state slot 0".into(),
                Err(error) => *status.borrow_mut() = format!("Load state failed: {error}"),
            }
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_save_sram_requested(move || {
            match core.borrow().save_save_ram() {
                Ok(path) => *status.borrow_mut() = format!("Saved SRAM: {}", path.display()),
                Err(error) => *status.borrow_mut() = format!("Save SRAM failed: {error}"),
            }
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_menu_back_requested(move || {
            if core.borrow_mut().pop_menu() {
                *status.borrow_mut() = "Menu back".into();
            }
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_show_home_requested(move || {
            core.borrow_mut().menu.clear_to_main();
            *status.borrow_mut() = "Home menu".into();
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_show_core_menu_requested(move || {
            core.borrow_mut().push_core_menu();
            *status.borrow_mut() = "Core menu".into();
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_show_content_menu_requested(move || {
            core.borrow_mut().push_content_menu();
            *status.borrow_mut() = "Content menu".into();
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_show_settings_menu_requested(move || {
            core.borrow_mut().push_settings_menu();
            *status.borrow_mut() = "Settings menu".into();
            refresh_weak(&ui_handle, &core, &status);
        });
    });

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_show_quick_menu_requested(move || {
            core.borrow_mut().push_quick_menu();
            *status.borrow_mut() = "Quick menu".into();
            refresh_weak(&ui_handle, &core, &status);
        });
    });
}

fn on<F>(window: &MainWindow, frontend: &SharedFrontend, status: &Rc<RefCell<String>>, f: F)
where
    F: FnOnce(MainWindow, SharedFrontend, Rc<RefCell<String>>),
{
    f(window.clone_strong(), frontend.clone(), status.clone());
}

fn refresh_weak(weak: &Weak<MainWindow>, frontend: &SharedFrontend, status: &Rc<RefCell<String>>) {
    if let Some(window) = weak.upgrade() {
        refresh_window(&window, frontend, status);
    }
}

fn run_frame_once(frontend: &SharedFrontend, status: &Rc<RefCell<String>>) {
    let result = frontend.borrow_mut().run_frame();
    if let Err(error) = result {
        *status.borrow_mut() = format!("Run frame failed: {error}");
    }
}

fn refresh_window(window: &MainWindow, frontend: &SharedFrontend, status: &Rc<RefCell<String>>) {
    let core = frontend.borrow();
    window.set_status_text(status.borrow().as_str().into());
    window.set_state_text(state_text(core.state()).into());
    window.set_current_core_text(current_core_text(&core).into());
    window.set_current_game_text(current_game_text(&core).into());
    window.set_cores(row_model(
        core.core_summaries().iter().map(core_row).collect(),
    ));
    window.set_games(row_model(
        core.game_summaries().iter().map(game_row).collect(),
    ));
    window.set_settings(row_model(
        core.setting_summaries().iter().map(setting_row).collect(),
    ));
    if let Some(menu) = core.current_menu_summary() {
        window.set_menu_title(menu.title.clone().into());
        window.set_menu_entries(row_model(menu_entries(&menu)));
    }
    let frame = core.gfx().last_frame();
    if frame.width > 0 && frame.height > 0 && !frame.rgba.is_empty() {
        let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            &frame.rgba,
            frame.width,
            frame.height,
        );
        window.set_video_frame(slint::Image::from_rgba8(buffer));
    }
}

fn row_model(rows: Vec<UiRow>) -> ModelRc<UiRow> {
    Rc::new(VecModel::from(rows)).into()
}

fn core_row(core: &UiCoreInfo) -> UiRow {
    UiRow {
        title: core.display_name.clone().into(),
        subtitle: format!(
            "{} • {}",
            core.system_name,
            core.path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
        )
        .into(),
    }
}

fn game_row(game: &UiGameEntry) -> UiRow {
    UiRow {
        title: game.label.clone().into(),
        subtitle: path_string(&game.path).into(),
    }
}

fn setting_row(setting: &UiSettingEntry) -> UiRow {
    UiRow {
        title: setting.key.clone().into(),
        subtitle: setting.value.clone().into(),
    }
}

fn menu_entries(menu: &UiMenuList) -> Vec<UiRow> {
    menu.entries
        .iter()
        .map(|entry| UiRow {
            title: entry.label.clone().into(),
            subtitle: if entry.value.is_empty() {
                entry.sublabel.clone()
            } else {
                format!("{} • {}", entry.sublabel, entry.value)
            }
            .into(),
        })
        .collect()
}

fn state_text(state: SessionState) -> &'static str {
    match state {
        SessionState::Empty => "No core loaded",
        SessionState::CoreLoaded => "Core loaded",
        SessionState::GameLoaded => "Game loaded",
    }
}

fn current_core_text(core: &FrontendCore) -> String {
    core.system_info()
        .map(|info| format!("Core: {} {}", info.library_name, info.library_version))
        .unwrap_or_else(|| "Core: none".into())
}

fn current_game_text(core: &FrontendCore) -> String {
    core.game_info()
        .map(|game| {
            format!(
                "Game: {}",
                game.path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
            )
        })
        .unwrap_or_else(|| "Game: none".into())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
