use retrofront_core::{
    FrontendCore, SessionState, UiCoreInfo, UiGameEntry, UiMenuList, UiSettingEntry,
};
use slint::{ComponentHandle, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel, Weak};
use std::cell::{Cell, RefCell};
use std::env;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;

slint::include_modules!();

type SharedFrontend = Rc<RefCell<FrontendCore>>;
static REFRESH_LAYOUT: OnceLock<StorageLayout> = OnceLock::new();

const DEFAULT_RETROPAD_OVERLAY: &str = r#"overlays = 2
overlay0_name = "portrait"
overlay0_full_screen = true
overlay0_normalized = true
overlay0_range_mod = 1.35
overlay0_alpha_mod = 0.72
overlay0_descs = 11
overlay0_desc0 = "dpad_area,0.18,0.78,rect,0.16,0.16"
overlay0_desc1 = "a,0.82,0.78,radial,0.07,0.07"
overlay0_desc2 = "b,0.68,0.83,radial,0.07,0.07"
overlay0_desc3 = "x,0.68,0.68,radial,0.07,0.07"
overlay0_desc4 = "y,0.82,0.63,radial,0.07,0.07"
overlay0_desc5 = "l,0.22,0.58,rect,0.12,0.045"
overlay0_desc6 = "r,0.78,0.58,rect,0.12,0.045"
overlay0_desc7 = "select,0.40,0.91,rect,0.08,0.04"
overlay0_desc8 = "start,0.60,0.91,rect,0.08,0.04"
overlay0_desc9 = "menu_toggle,0.50,0.58,rect,0.08,0.04"
overlay0_desc10 = "overlay_next,0.93,0.08,rect,0.045,0.045"
overlay1_name = "landscape"
overlay1_full_screen = true
overlay1_normalized = true
overlay1_range_mod = 1.25
overlay1_alpha_mod = 0.62
overlay1_descs = 11
overlay1_desc0 = "dpad_area,0.13,0.72,rect,0.12,0.18"
overlay1_desc1 = "a,0.88,0.72,radial,0.055,0.08"
overlay1_desc2 = "b,0.78,0.82,radial,0.055,0.08"
overlay1_desc3 = "x,0.78,0.62,radial,0.055,0.08"
overlay1_desc4 = "y,0.88,0.52,radial,0.055,0.08"
overlay1_desc5 = "l,0.18,0.12,rect,0.13,0.055"
overlay1_desc6 = "r,0.82,0.12,rect,0.13,0.055"
overlay1_desc7 = "select,0.42,0.90,rect,0.06,0.04"
overlay1_desc8 = "start,0.58,0.90,rect,0.06,0.04"
overlay1_desc9 = "menu_toggle,0.50,0.10,rect,0.06,0.04"
overlay1_desc10 = "overlay_next,0.96,0.08,rect,0.035,0.05"
"#;

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
    thumbnails_dir: PathBuf,
    database_dir: PathBuf,
    cheat_dir: PathBuf,
    remaps_dir: PathBuf,
    shaders_dir: PathBuf,
    autoconfig_dir: PathBuf,
    logs_dir: PathBuf,
    records_dir: PathBuf,
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
            thumbnails_dir: root.join("thumbnails"),
            database_dir: root.join("database/rdb"),
            cheat_dir: root.join("cht"),
            remaps_dir: root.join("remaps"),
            shaders_dir: root.join("shaders"),
            autoconfig_dir: root.join("autoconfig"),
            logs_dir: root.join("logs"),
            records_dir: root.join("records"),
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
            &self.thumbnails_dir,
            &self.database_dir,
            &self.cheat_dir,
            &self.remaps_dir,
            &self.shaders_dir,
            &self.autoconfig_dir,
            &self.logs_dir,
            &self.records_dir,
            self.overlay_config.parent().unwrap_or(&self.overlays_dir),
        ] {
            std::fs::create_dir_all(directory)
                .map_err(|error| format!("create {}: {error}", directory.display()))?;
        }
        if !cfg!(target_os = "ios") {
            std::fs::create_dir_all(&self.core_dir)
                .map_err(|error| format!("create {}: {error}", self.core_dir.display()))?;
        }
        self.ensure_default_overlay()?;
        Ok(())
    }

    fn ensure_default_overlay(&self) -> Result<(), String> {
        if self.overlay_config.exists() {
            return Ok(());
        }
        if let Some(parent) = self.overlay_config.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("create {}: {error}", parent.display()))?;
        }
        std::fs::write(&self.overlay_config, DEFAULT_RETROPAD_OVERLAY)
            .map_err(|error| format!("write {}: {error}", self.overlay_config.display()))
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
        frontend.set_setting(
            "thumbnails_directory",
            path_string(&self.thumbnails_dir).as_str(),
        );
        frontend.set_setting(
            "content_database_path",
            path_string(&self.database_dir).as_str(),
        );
        frontend.set_setting("cheat_database_path", path_string(&self.cheat_dir).as_str());
        frontend.set_setting(
            "input_remapping_directory",
            path_string(&self.remaps_dir).as_str(),
        );
        frontend.set_setting("video_shader_dir", path_string(&self.shaders_dir).as_str());
        frontend.set_setting(
            "joypad_autoconfig_dir",
            path_string(&self.autoconfig_dir).as_str(),
        );
        frontend.set_setting("log_dir", path_string(&self.logs_dir).as_str());
        frontend.set_setting(
            "recording_output_directory",
            path_string(&self.records_dir).as_str(),
        );
        if cfg!(target_os = "ios") {
            frontend.set_setting("video_driver", "metal");
            frontend.set_setting("video_bgfx_renderer", "metal");
            frontend.set_setting("input_driver", "apple_gamecontroller");
        } else {
            frontend.set_setting("video_driver", "glcore");
            frontend.set_setting("video_bgfx_renderer", "opengl");
            frontend.set_setting("input_driver", "udev");
        }
        frontend.set_info_dir(&self.info_dir);
        Ok(())
    }
    fn ui_directory_rows(&self) -> Vec<UiSettingEntry> {
        [
            ("Root", &self.root),
            ("Config", &self.config_file),
            ("Cores", &self.core_dir),
            ("Content", &self.content_dir),
            ("Assets", &self.assets_dir),
            ("Core info", &self.info_dir),
            ("Overlays", &self.overlays_dir),
            ("Saves", &self.saves_dir),
            ("States", &self.states_dir),
            ("System", &self.system_dir),
            ("Screenshots", &self.screenshots_dir),
            ("Playlists", &self.playlists_dir),
            ("Cache", &self.cache_dir),
            ("Thumbnails", &self.thumbnails_dir),
            ("Database", &self.database_dir),
            ("Cheats", &self.cheat_dir),
            ("Remaps", &self.remaps_dir),
            ("Shaders", &self.shaders_dir),
            ("Autoconfig", &self.autoconfig_dir),
            ("Logs", &self.logs_dir),
            ("Records", &self.records_dir),
        ]
        .into_iter()
        .map(|(key, path)| UiSettingEntry {
            key: key.to_string(),
            value: path_string(path),
        })
        .collect()
    }
}

fn layout_for_refresh() -> StorageLayout {
    REFRESH_LAYOUT
        .get()
        .cloned()
        .unwrap_or_else(StorageLayout::current)
}

fn feature_rows(core: &FrontendCore, layout: &StorageLayout) -> Vec<UiSettingEntry> {
    let core_count = core.core_summaries().len();
    let game_count = core.game_summaries().len();
    let runtime = core.runtime_summaries();
    vec![
        UiSettingEntry { key: "iOS launch backend".into(), value: "Slint winit + software renderer selected by ios feature; SwiftUI launch surface removed and replaced with Rust-owned UI".into() },
        UiSettingEntry { key: "Linux launch backend".into(), value: "The desktop binary keeps the same Slint file and Rust callback graph through winit + femtovg".into() },
        UiSettingEntry { key: "SwiftUI hero parity".into(), value: "Hero, live status, current state, current core/game and refresh controls are exposed as Slint properties".into() },
        UiSettingEntry { key: "SwiftUI metrics parity".into(), value: "State, core count, game count, settings count and API count are live Rust-backed metric cards".into() },
        UiSettingEntry { key: "SwiftUI command parity".into(), value: "Run frame, reset, save state, load state, SRAM save and stop all execute from Slint callbacks".into() },
        UiSettingEntry { key: "Core policy".into(), value: format!("{core_count} discovered cores; archifacts/ios copying remains unfiltered") },
        UiSettingEntry { key: "Library".into(), value: format!("{game_count} scanned entries from {}", layout.content_dir.display()) },
        UiSettingEntry { key: "Files app storage".into(), value: "iOS Documents/RetroArch/Roms mirrors the previous Files drop-zone and Linux uses XDG config storage".into() },
        UiSettingEntry { key: "Save data".into(), value: "SRAM, savestates, save/system directories and RetroArch-compatible config paths".into() },
        UiSettingEntry { key: "Input".into(), value: "Joypad bitmasks, keyboard callback, descriptors, max users, rumble, sensors, overlays, MFi/GameController link".into() },
        UiSettingEntry { key: "Media".into(), value: "Software frame ingest, Slint image viewport, audio samples/batches, frame stepping, playback timer and screenshots directory".into() },
        UiSettingEntry { key: "Menu activation".into(), value: "RetroArch-style core/content/settings/quick-menu rows are clickable and execute Rust menu actions".into() },
        UiSettingEntry { key: "Environment".into(), value: "Core options v0/v1/v2/intl, VFS v4, directories, messages, geometry, rotation, perf, MIDI, location/camera stubs".into() },
        UiSettingEntry { key: "Runtime telemetry".into(), value: format!("{} rows exported to the dashboard", runtime.len()) },
    ]
}

pub fn run() -> Result<(), slint::PlatformError> {
    configure_slint_backend();
    let layout = StorageLayout::current();
    let _ = REFRESH_LAYOUT.set(layout.clone());
    let frontend = Rc::new(RefCell::new(FrontendCore::new()));
    let status = Rc::new(RefCell::new(String::from("Ready")));

    initialize_frontend(&frontend, &layout, &status);

    let window = MainWindow::new()?;
    window.set_mobile_layout(cfg!(target_os = "ios"));
    refresh_window(&window, &frontend, &status);
    wire_callbacks(&window, frontend, layout, status);
    window.run()
}

#[no_mangle]
pub extern "C" fn retrofront_slint_ios_main() -> i32 {
    run_main()
}

pub fn run_main() -> i32 {
    match run() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("Retrofront Slint UI failed: {error}");
            1
        }
    }
}

fn configure_slint_backend() {
    if cfg!(target_os = "ios") {
        env::set_var("SLINT_BACKEND", "winit");
        env::set_var("SLINT_RENDERER", "skia");
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

    on(window, &frontend, &status, |ui, core, status| {
        let ui_handle = ui.as_weak();
        ui.on_activate_menu_entry_requested(move |index| {
            let action_id = core.borrow().current_menu_summary().and_then(|menu| {
                menu.entries
                    .get(index as usize)
                    .map(|entry| entry.action_id)
            });
            match action_id {
                Some(action_id) if core.borrow_mut().activate_menu_action(action_id) => {
                    *status.borrow_mut() = format!("Activated menu action {action_id}");
                }
                Some(action_id) => {
                    *status.borrow_mut() = format!("Menu action {action_id} is not available");
                }
                None => *status.borrow_mut() = "Selected menu row is no longer available".into(),
            }
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

    let layout = layout_for_refresh();
    let core_rows: Vec<_> = core.core_summaries().iter().map(core_row).collect();
    let game_rows: Vec<_> = core.game_summaries().iter().map(game_row).collect();
    let setting_rows: Vec<_> = core.setting_summaries().iter().map(setting_row).collect();
    let runtime_rows: Vec<_> = core.runtime_summaries().iter().map(setting_row).collect();
    let api_rows: Vec<_> = core
        .libretro_api_summaries()
        .iter()
        .map(setting_row)
        .collect();
    let directory_rows: Vec<_> = layout.ui_directory_rows().iter().map(setting_row).collect();
    let feature_rows: Vec<_> = feature_rows(&core, &layout)
        .iter()
        .map(setting_row)
        .collect();

    window.set_cores_count_text(core_rows.len().to_string().into());
    window.set_games_count_text(game_rows.len().to_string().into());
    window.set_settings_count_text(setting_rows.len().to_string().into());
    window.set_runtime_count_text(runtime_rows.len().to_string().into());
    window.set_api_count_text(api_rows.len().to_string().into());
    window.set_storage_count_text(directory_rows.len().to_string().into());
    window.set_feature_count_text(feature_rows.len().to_string().into());
    window.set_shell_mode_text(shell_mode_text().into());
    window.set_library_hint_text(library_hint(&layout).into());
    window.set_core_hint_text(core_hint(&layout).into());

    window.set_cores(row_model(core_rows));
    window.set_games(row_model(game_rows));
    window.set_settings(row_model(setting_rows));
    window.set_runtime_stats(row_model(runtime_rows));
    window.set_api_rows(row_model(api_rows));
    window.set_directory_rows(row_model(directory_rows));
    window.set_feature_rows(row_model(feature_rows));
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

fn shell_mode_text() -> &'static str {
    if cfg!(target_os = "ios") {
        "Slint iOS shell"
    } else {
        "Slint Linux shell"
    }
}

fn library_hint(layout: &StorageLayout) -> String {
    if cfg!(target_os = "ios") {
        format!(
            "Drop ROMs in Files at On My iPhone/Retrofront/{}",
            layout
                .content_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Roms")
        )
    } else {
        format!("Scan ROMs from {}", layout.content_dir.display())
    }
}

fn core_hint(layout: &StorageLayout) -> String {
    if cfg!(target_os = "ios") {
        "Bundled Frameworks are scanned at startup and copied without filtering".into()
    } else {
        format!("Place libretro cores in {}", layout.core_dir.display())
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
