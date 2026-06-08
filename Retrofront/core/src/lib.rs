//! Retrofront Rust management core.

mod core_info;
mod dylib;
pub mod gfx;
mod launch;
pub mod libretro;
mod menu;
mod options;
mod playlist;
mod scanner;
mod settings;

use dylib::Library;
use gfx::{GfxBackendKind, GfxRuntime, HardwareRenderRequest, PixelFormat};
use launch::LaunchDecisionKind;
use options::CoreOptionsManager;
pub use options::{CoreOptionDefinition, CoreOptionValue};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uint, c_void};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::{Mutex, OnceLock};

pub const RETRO_HW_FRAME_BUFFER_VALID: *const c_void = usize::MAX as *const c_void;

macro_rules! sym {
    ($lib:expr, $name:literal, $ty:ty) => {{
        let name =
            CStr::from_bytes_with_nul(concat!($name, "\0").as_bytes()).expect("static symbol name");
        $lib.symbol::<$ty>(name)?.get()
    }};
}

type RetroSetEnvironment = unsafe extern "C" fn(libretro::retro_environment_t);
type RetroSetVideoRefresh = unsafe extern "C" fn(libretro::retro_video_refresh_t);
type RetroSetAudioSample = unsafe extern "C" fn(libretro::retro_audio_sample_t);
type RetroSetAudioSampleBatch = unsafe extern "C" fn(libretro::retro_audio_sample_batch_t);
type RetroSetInputPoll = unsafe extern "C" fn(libretro::retro_input_poll_t);
type RetroSetInputState = unsafe extern "C" fn(libretro::retro_input_state_t);
type RetroInit = unsafe extern "C" fn();
type RetroDeinit = unsafe extern "C" fn();
type RetroApiVersion = unsafe extern "C" fn() -> c_uint;
type RetroGetSystemInfo = unsafe extern "C" fn(*mut libretro::retro_system_info);
type RetroGetSystemAvInfo = unsafe extern "C" fn(*mut libretro::retro_system_av_info);
type RetroLoadGame = unsafe extern "C" fn(*const libretro::retro_game_info) -> bool;
type RetroUnloadGame = unsafe extern "C" fn();
type RetroRun = unsafe extern "C" fn();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    pub library_name: String,
    pub library_version: String,
    pub valid_extensions: Vec<String>,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInfo {
    pub path: PathBuf,
    pub meta: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendEvent {
    VideoFrame {
        width: u32,
        height: u32,
        pitch: usize,
        pixel_format: u32,
        frame_number: u64,
    },
    AudioBatch {
        frames: usize,
    },
    AudioSample {
        left: i16,
        right: i16,
    },
    EnvironmentCommand {
        command: u32,
        handled: bool,
    },
    InputPoll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Empty,
    CoreLoaded,
    GameLoaded,
}

struct CoreApi {
    _library: Library,
    retro_set_environment: RetroSetEnvironment,
    retro_set_video_refresh: RetroSetVideoRefresh,
    retro_set_audio_sample: RetroSetAudioSample,
    retro_set_audio_sample_batch: RetroSetAudioSampleBatch,
    retro_set_input_poll: RetroSetInputPoll,
    retro_set_input_state: RetroSetInputState,
    retro_init: RetroInit,
    retro_deinit: RetroDeinit,
    #[allow(dead_code)]
    retro_api_version: RetroApiVersion,
    retro_get_system_info: RetroGetSystemInfo,
    retro_get_system_av_info: RetroGetSystemAvInfo,
    retro_load_game: RetroLoadGame,
    retro_unload_game: RetroUnloadGame,
    retro_run: RetroRun,
}

impl CoreApi {
    fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let lib = Library::open(path.as_ref()).map_err(|e| e.to_string())?;
        unsafe {
            Ok(Self {
                retro_set_environment: sym!(lib, "retro_set_environment", RetroSetEnvironment),
                retro_set_video_refresh: sym!(lib, "retro_set_video_refresh", RetroSetVideoRefresh),
                retro_set_audio_sample: sym!(lib, "retro_set_audio_sample", RetroSetAudioSample),
                retro_set_audio_sample_batch: sym!(
                    lib,
                    "retro_set_audio_sample_batch",
                    RetroSetAudioSampleBatch
                ),
                retro_set_input_poll: sym!(lib, "retro_set_input_poll", RetroSetInputPoll),
                retro_set_input_state: sym!(lib, "retro_set_input_state", RetroSetInputState),
                retro_init: sym!(lib, "retro_init", RetroInit),
                retro_deinit: sym!(lib, "retro_deinit", RetroDeinit),
                retro_api_version: sym!(lib, "retro_api_version", RetroApiVersion),
                retro_get_system_info: sym!(lib, "retro_get_system_info", RetroGetSystemInfo),
                retro_get_system_av_info: sym!(
                    lib,
                    "retro_get_system_av_info",
                    RetroGetSystemAvInfo
                ),
                retro_load_game: sym!(lib, "retro_load_game", RetroLoadGame),
                retro_unload_game: sym!(lib, "retro_unload_game", RetroUnloadGame),
                retro_run: sym!(lib, "retro_run", RetroRun),
                _library: lib,
            })
        }
    }
}

pub struct FrontendCore {
    api: Option<CoreApi>,
    system_info: Option<SystemInfo>,
    game: Option<GameInfo>,
    events: VecDeque<FrontendEvent>,
    joypad_buttons: [i16; 16],
    pub gfx: GfxRuntime,
    pub options: CoreOptionsManager,
    pub core_info: core_info::CoreInfoList,
    pub settings: settings::Settings,
    pub menu: menu::MenuEngine,
    pub scanner: scanner::Scanner,
    pub launcher: launch::LaunchManager,
}

static CORE_INSTANCE: OnceLock<Mutex<FrontendCore>> = OnceLock::new();

thread_local! {
    static ACTIVE_CORE: RefCell<*mut FrontendCore> = RefCell::new(ptr::null_mut());
}

pub fn with_active_frontend<F, R>(f: F) -> R
where
    F: FnOnce(&mut FrontendCore) -> R,
{
    ACTIVE_CORE.with(|active| {
        let current_ptr = *active.borrow();
        if !current_ptr.is_null() {
            // Safety: current_ptr is only set while the CORE_INSTANCE mutex is held on this thread.
            return f(unsafe { &mut *current_ptr });
        }

        let mutex = CORE_INSTANCE.get_or_init(|| Mutex::new(FrontendCore::new()));
        let mut core = mutex.lock().unwrap();

        let core_ptr = &mut *core as *mut FrontendCore;
        *active.borrow_mut() = core_ptr;
        let res = f(&mut core);
        *active.borrow_mut() = ptr::null_mut();
        res
    })
}

impl FrontendCore {
    pub fn new() -> Self {
        Self {
            api: None,
            system_info: None,
            game: None,
            events: VecDeque::new(),
            joypad_buttons: [0; 16],
            gfx: GfxRuntime::new(),
            options: CoreOptionsManager::new(),
            core_info: core_info::CoreInfoList::new(),
            settings: settings::Settings::new(),
            menu: menu::MenuEngine::new(),
            scanner: scanner::Scanner::new(),
            launcher: launch::LaunchManager::new(),
        }
    }

    pub fn state(&self) -> SessionState {
        if self.game.is_some() {
            SessionState::GameLoaded
        } else if self.api.is_some() {
            SessionState::CoreLoaded
        } else {
            SessionState::Empty
        }
    }

    pub fn system_info(&self) -> Option<&SystemInfo> {
        self.system_info.as_ref()
    }
    pub fn game_info(&self) -> Option<&GameInfo> {
        self.game.as_ref()
    }
    pub fn last_error(&self) -> Option<&str> {
        None
    }
    pub fn next_event(&mut self) -> Option<FrontendEvent> {
        self.events.pop_front()
    }

    pub fn gfx(&self) -> &GfxRuntime {
        &self.gfx
    }

    pub fn set_gfx_backend(&mut self, backend: GfxBackendKind) {
        self.gfx.set_backend(backend);
    }

    pub fn configure_from_settings(&mut self) {
        self.settings.ensure_directories();
        self.core_info
            .set_info_dir(self.settings.libretro_info_path());
        self.options.set_config_path(
            self.settings
                .path_value("core_options_path")
                .unwrap_or_else(|| self.settings.base_dir.join("retroarch-core-options.cfg")),
        );
    }

    pub fn load_core(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        self.unload_game();
        if let Some(api) = self.api.as_ref() {
            unsafe { (api.retro_deinit)() };
            self.api = None;
            self.system_info = None;
        }

        let api = CoreApi::load(path)?;
        unsafe {
            (api.retro_set_environment)(Some(Self::retro_environment_callback));
            (api.retro_set_video_refresh)(Some(Self::retro_video_refresh_callback));
            (api.retro_set_audio_sample)(Some(Self::retro_audio_sample_callback));
            (api.retro_set_audio_sample_batch)(Some(Self::retro_audio_sample_batch_callback));
            (api.retro_set_input_poll)(Some(Self::retro_input_poll_callback));
            (api.retro_set_input_state)(Some(Self::retro_input_state_callback));
            (api.retro_init)();
        }

        let mut sys_info = libretro::retro_system_info {
            library_name: ptr::null(),
            library_version: ptr::null(),
            valid_extensions: ptr::null(),
            need_fullpath: false,
            block_extract: false,
        };
        unsafe { (api.retro_get_system_info)(&mut sys_info) };

        self.system_info = Some(SystemInfo {
            library_name: unsafe { CStr::from_ptr(sys_info.library_name) }
                .to_string_lossy()
                .into_owned(),
            library_version: unsafe { CStr::from_ptr(sys_info.library_version) }
                .to_string_lossy()
                .into_owned(),
            valid_extensions: unsafe { CStr::from_ptr(sys_info.valid_extensions) }
                .to_string_lossy()
                .split('|')
                .map(|s| s.to_string())
                .collect(),
            need_fullpath: sys_info.need_fullpath,
            block_extract: sys_info.block_extract,
        });

        self.api = Some(api);
        Ok(())
    }

    pub fn load_game(
        &mut self,
        path: impl AsRef<Path>,
        meta: Option<String>,
    ) -> Result<(), String> {
        let path_buf = path.as_ref().to_path_buf();
        let c_path = CString::new(path_buf.to_string_lossy().as_bytes()).unwrap();

        self.unload_game();

        let Some(api) = self.api.as_ref() else {
            return Err("no core loaded".to_string());
        };

        let game_info = libretro::retro_game_info {
            path: c_path.as_ptr(),
            data: ptr::null(),
            size: 0,
            meta: ptr::null(),
        };

        if unsafe { (api.retro_load_game)(&game_info) } {
            self.game = Some(GameInfo {
                path: path_buf,
                meta,
            });

            let mut av_info = libretro::retro_system_av_info {
                geometry: libretro::retro_game_geometry {
                    base_width: 0,
                    base_height: 0,
                    max_width: 0,
                    max_height: 0,
                    aspect_ratio: 0.0,
                },
                timing: libretro::retro_system_timing {
                    fps: 0.0,
                    sample_rate: 0.0,
                },
            };
            unsafe { (api.retro_get_system_av_info)(&mut av_info) };
            self.gfx.update_system_av_info(&av_info);

            Ok(())
        } else {
            Err("core failed to load game".to_string())
        }
    }

    pub fn launch_content(
        &mut self,
        path: impl AsRef<Path>,
        requested_core: Option<PathBuf>,
        meta: Option<String>,
    ) -> Result<launch::LaunchPlan, String> {
        if self.core_info.cores.is_empty() {
            let dir = self.settings.libretro_directory();
            self.core_info.scan_directory(&dir);
        }

        let content_path = path.as_ref().to_path_buf();
        let plan = self.launcher.plan_content_launch(
            &content_path,
            &self.core_info,
            &self.settings,
            requested_core.as_deref(),
        );

        match (&plan.decision, plan.selected_core.clone()) {
            (LaunchDecisionKind::Selected, Some(core_path)) => {
                self.load_core(core_path)?;
                self.load_game(&content_path, meta)?;
                Ok(plan)
            }
            (LaunchDecisionKind::NeedsCoreChoice, _) => Err(plan.reason.clone()),
            (LaunchDecisionKind::NoCore, _) => Err(plan.reason.clone()),
            _ => Err("invalid launch plan".to_string()),
        }
    }

    pub fn unload_game(&mut self) {
        if let Some(api) = self.api.as_ref() {
            if self.game.is_some() {
                unsafe { (api.retro_unload_game)() };
                self.game = None;
            }
        }
    }

    pub fn run_frame(&mut self) -> Result<(), String> {
        let Some(api) = self.api.as_ref() else {
            return Err("no core loaded".to_string());
        };
        if self.game.is_none() {
            return Err("no game loaded".to_string());
        }

        unsafe { (api.retro_run)() };
        Ok(())
    }

    pub fn joypad_button(&self, id: u32) -> i16 {
        if id < 16 {
            self.joypad_buttons[id as usize]
        } else {
            0
        }
    }

    pub fn set_joypad_button(&mut self, id: u32, pressed: bool) -> Result<(), String> {
        if id < 16 {
            self.joypad_buttons[id as usize] = if pressed { 1 } else { 0 };
            Ok(())
        } else {
            Err("invalid button id".to_string())
        }
    }

    unsafe extern "C" fn retro_environment_callback(command: c_uint, data: *mut c_void) -> bool {
        with_active_frontend(|core| {
            let res = match command {
                libretro::RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
                    let format = unsafe { *(data as *const libretro::retro_pixel_format) };
                    if let Some(p) = PixelFormat::from_libretro(format) {
                        core.gfx.set_pixel_format(p);
                    }
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_VARIABLE => {
                    let var = unsafe { &mut *(data as *mut libretro::retro_variable) };
                    let key = unsafe { CStr::from_ptr(var.key) }.to_string_lossy();
                    let val_ptr = core.options.get_variable_ptr(&key);
                    var.value = val_ptr;
                    !val_ptr.is_null()
                }
                libretro::RETRO_ENVIRONMENT_SET_VARIABLES => {
                    core.options
                        .set_definitions_v0(data as *const libretro::retro_variable);
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_CORE_OPTIONS => {
                    core.options
                        .set_definitions_v1(data as *const libretro::retro_core_option_definition);
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2 => {
                    let v2 = unsafe { &*(data as *const libretro::retro_core_options_v2) };
                    core.options
                        .set_definitions_v2(v2.definitions, v2.categories);
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL => {
                    let intl = unsafe { &*(data as *const libretro::retro_core_options_v2_intl) };
                    let us = unsafe { &*intl.us };
                    core.options
                        .set_definitions_v2(us.definitions, us.categories);
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE => {
                    unsafe { *(data as *mut bool) = core.options.check_updated() };
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_HW_RENDER => {
                    let req = unsafe { &*(data as *const libretro::retro_hw_render_callback) };
                    core.gfx
                        .set_hardware_render_request(HardwareRenderRequest::from_libretro(req));
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_CAN_DUPE => {
                    unsafe { *(data as *mut bool) = true };
                    true
                }
                _ => false,
            };
            core.events.push_back(FrontendEvent::EnvironmentCommand {
                command,
                handled: res,
            });
            res
        })
    }

    unsafe extern "C" fn retro_video_refresh_callback(
        data: *const c_void,
        width: c_uint,
        height: c_uint,
        pitch: usize,
    ) {
        with_active_frontend(|core| {
            if !data.is_null() && data != RETRO_HW_FRAME_BUFFER_VALID {
                let _ = core
                    .gfx
                    .ingest_software_frame(data.cast(), width, height, pitch);
            }
            core.events.push_back(FrontendEvent::VideoFrame {
                width,
                height,
                pitch,
                pixel_format: core.gfx.last_frame().source_format.code(),
                frame_number: core.gfx.frame_counter(),
            });
        });
    }

    unsafe extern "C" fn retro_audio_sample_callback(left: i16, right: i16) {
        with_active_frontend(|core| {
            core.events
                .push_back(FrontendEvent::AudioSample { left, right });
        });
    }

    unsafe extern "C" fn retro_audio_sample_batch_callback(
        _data: *const i16,
        frames: usize,
    ) -> usize {
        with_active_frontend(|core| {
            core.events.push_back(FrontendEvent::AudioBatch { frames });
        });
        frames
    }

    unsafe extern "C" fn retro_input_poll_callback() {
        with_active_frontend(|core| {
            core.events.push_back(FrontendEvent::InputPoll);
        });
    }

    unsafe extern "C" fn retro_input_state_callback(
        _port: c_uint,
        _device: c_uint,
        _index: c_uint,
        id: c_uint,
    ) -> i16 {
        with_active_frontend(|core| core.joypad_button(id))
    }
}

#[repr(C)]
pub struct RfFrontend {
    last_error: CString,
    info_name: CString,
    info_version: CString,
    info_extensions: CString,
    cached_options: Vec<CString>,
    cached_option_values: Vec<Vec<RfCoreOptionValue>>,
    cached_cores: Vec<RfCoreInfo>,
    cached_menu_entries: Vec<RfMenuEntry>,
    cached_strings: Vec<CString>,
}

#[repr(C)]
pub struct RfSystemInfo {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

#[repr(C)]
pub struct RfEvent {
    pub kind: u32,
    pub a: u64,
    pub b: u64,
    pub c: u64,
}

#[repr(C)]
pub struct RfVideoFrameInfo {
    pub width: u32,
    pub height: u32,
    pub pitch: u64,
    pub rgba_len: u64,
    pub pixel_format: u32,
    pub frame_number: u64,
}

#[repr(C)]
pub struct RfGfxVideoConfig {
    pub base_width: u32,
    pub base_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub aspect_ratio: f32,
    pub output_width: u32,
    pub output_height: u32,
    pub scale_mode: u32,
    pub filter_mode: u32,
    pub rotation_quarters: u32,
    pub vsync: bool,
}

#[repr(C)]
pub struct RfGfxHostHandles {
    pub native_view: u64,
    pub context: u64,
    pub framebuffer: usize,
    pub render_callback: *const c_void,
    pub get_proc_address: *const c_void,
    pub user_data: *mut c_void,
}

#[repr(C)]
pub struct RfGfxDriverInfo {
    pub backend: u32,
    pub frame_number: u64,
    pub hardware_ready: bool,
    pub rendered: bool,
}

#[repr(C)]
pub struct RfCoreOptionValue {
    pub value: *const c_char,
    pub label: *const c_char,
}

#[repr(C)]
pub struct RfCoreOption {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub info: *const c_char,
    pub value: *const c_char,
    pub values: *const RfCoreOptionValue,
    pub values_count: usize,
}

#[repr(C)]
pub struct RfCoreInfo {
    pub path: *const c_char,
    pub display_name: *const c_char,
    pub system_name: *const c_char,
    pub supported_extensions: *const c_char,
}

#[repr(C)]
pub struct RfGameEntry {
    pub path: *const c_char,
    pub label: *const c_char,
}

#[repr(C)]
pub struct RfMenuEntry {
    pub label: *const c_char,
    pub sublabel: *const c_char,
    pub kind: u32,
    pub value: *const c_char,
    pub action_id: u32,
}

#[repr(C)]
pub struct RfMenuList {
    pub title: *const c_char,
    pub entry_count: usize,
}

#[repr(C)]
pub struct RfSettingEntry {
    pub key: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
pub struct RfLaunchPlan {
    pub content_path: *const c_char,
    pub content_extension: *const c_char,
    pub decision: u32,
    pub selected_core_path: *const c_char,
    pub candidate_count: usize,
    pub reason: *const c_char,
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_create() -> *mut RfFrontend {
    Box::into_raw(Box::new(RfFrontend {
        last_error: CString::default(),
        info_name: CString::default(),
        info_version: CString::default(),
        info_extensions: CString::default(),
        cached_options: Vec::new(),
        cached_option_values: Vec::new(),
        cached_cores: Vec::new(),
        cached_menu_entries: Vec::new(),
        cached_strings: Vec::new(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_destroy(frontend: *mut RfFrontend) {
    if !frontend.is_null() {
        unsafe { drop(Box::from_raw(frontend)) };
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_state(_frontend: *const RfFrontend) -> u32 {
    with_active_frontend(|core| match core.state() {
        SessionState::Empty => 0,
        SessionState::CoreLoaded => 1,
        SessionState::GameLoaded => 2,
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_load_core(
    frontend: *mut RfFrontend,
    path: *const c_char,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(path_str) = ptr_to_str(path) else {
        return false;
    };
    let res = with_active_frontend(|core| core.load_core(path_str));
    match res {
        Ok(()) => {
            frontend.last_error = CString::default();
            let sys_info = with_active_frontend(|core| core.system_info().cloned());
            if let Some(info) = sys_info {
                cache_system_info(frontend, &info);
            }
            true
        }
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_load_game(
    frontend: *mut RfFrontend,
    path: *const c_char,
    meta: *const c_char,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(path_str) = ptr_to_str(path) else {
        return false;
    };
    let meta_str = ptr_to_str(meta);
    let res = with_active_frontend(|core| core.load_game(path_str, meta_str));
    match res {
        Ok(()) => {
            frontend.last_error = CString::default();
            true
        }
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_launch_content(
    frontend: *mut RfFrontend,
    path: *const c_char,
    preferred_core: *const c_char,
    meta: *const c_char,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(path_str) = ptr_to_str(path) else {
        return false;
    };
    let preferred = ptr_to_str(preferred_core).map(PathBuf::from);
    let meta_str = ptr_to_str(meta);
    let res = with_active_frontend(|core| core.launch_content(path_str, preferred, meta_str));
    match res {
        Ok(_) => {
            frontend.last_error = CString::default();
            let sys_info = with_active_frontend(|core| core.system_info().cloned());
            if let Some(info) = sys_info {
                cache_system_info(frontend, &info);
            }
            true
        }
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_run_frame(frontend: *mut RfFrontend) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let res = with_active_frontend(|core| core.run_frame());
    match res {
        Ok(()) => {
            frontend.last_error = CString::default();
            true
        }
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_unload_game(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| core.unload_game());
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_gfx_backend(
    _frontend: *mut RfFrontend,
    backend: u32,
) -> bool {
    let Some(kind) = GfxBackendKind::from_code(backend) else {
        return false;
    };
    with_active_frontend(|core| core.set_gfx_backend(kind));
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_get_gfx_video_config(
    _frontend: *const RfFrontend,
    out_config: *mut RfGfxVideoConfig,
) -> bool {
    let Some(out_config) = (unsafe { out_config.as_mut() }) else {
        return false;
    };
    with_active_frontend(|core| {
        let config = core.gfx.video_config();
        out_config.base_width = config.base_width;
        out_config.base_height = config.base_height;
        out_config.max_width = config.max_width;
        out_config.max_height = config.max_height;
        out_config.aspect_ratio = config.aspect_ratio;
        out_config.output_width = config.output_width;
        out_config.output_height = config.output_height;
        out_config.scale_mode = config.scale_mode as u32;
        out_config.filter_mode = config.filter_mode as u32;
        out_config.rotation_quarters = config.rotation_quarters;
        out_config.vsync = config.vsync;
        true
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_gfx_video_config(
    _frontend: *mut RfFrontend,
    config: *const RfGfxVideoConfig,
) -> bool {
    let Some(config) = (unsafe { config.as_ref() }) else {
        return false;
    };
    with_active_frontend(|core| {
        core.gfx.set_video_config(gfx::GfxVideoConfig {
            base_width: config.base_width,
            base_height: config.base_height,
            max_width: config.max_width,
            max_height: config.max_height,
            aspect_ratio: config.aspect_ratio,
            output_width: config.output_width,
            output_height: config.output_height,
            scale_mode: gfx::GfxScaleMode::from_code(config.scale_mode).unwrap_or_default(),
            filter_mode: gfx::GfxFilterMode::from_code(config.filter_mode).unwrap_or_default(),
            rotation_quarters: config.rotation_quarters,
            vsync: config.vsync,
        });
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_gfx_host_handles(
    _frontend: *mut RfFrontend,
    handles: *const RfGfxHostHandles,
) -> bool {
    let Some(handles) = (unsafe { handles.as_ref() }) else {
        return false;
    };
    with_active_frontend(|core| {
        core.gfx.set_host_handles(gfx::HostRenderHandles {
            native_view: handles.native_view,
            context: handles.context,
            framebuffer: handles.framebuffer,
            render_callback: if handles.render_callback.is_null() {
                None
            } else {
                Some(unsafe { std::mem::transmute(handles.render_callback) })
            },
            get_proc_address: if handles.get_proc_address.is_null() {
                None
            } else {
                Some(unsafe { std::mem::transmute(handles.get_proc_address) })
            },
            user_data: handles.user_data,
        });
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_joypad_button(
    frontend: *mut RfFrontend,
    button_id: u32,
    pressed: bool,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let res = with_active_frontend(|core| core.set_joypad_button(button_id, pressed));
    match res {
        Ok(()) => {
            frontend.last_error = CString::default();
            true
        }
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_gfx_driver_info(
    _frontend: *const RfFrontend,
    out: *mut RfGfxDriverInfo,
) -> bool {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    with_active_frontend(|core| {
        let status = core.gfx.driver_status();
        let last_present = status.last_present.as_ref();
        *out = RfGfxDriverInfo {
            backend: last_present.map_or(0, |p| p.backend as u32),
            frame_number: status.frame_counter,
            hardware_ready: status.hardware_ready,
            rendered: last_present.is_some_and(|p| p.rendered),
        };
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_video_frame_info(
    _frontend: *const RfFrontend,
    out: *mut RfVideoFrameInfo,
) -> bool {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    with_active_frontend(|core| {
        let frame = core.gfx.last_frame();
        if frame.width == 0 {
            return false;
        }
        *out = RfVideoFrameInfo {
            width: frame.width,
            height: frame.height,
            pitch: frame.pitch as u64,
            rgba_len: frame.rgba.len() as u64,
            pixel_format: frame.source_format.code(),
            frame_number: core.gfx.frame_counter(),
        };
        true
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_copy_video_frame_rgba(
    _frontend: *const RfFrontend,
    out_rgba: *mut u8,
    out_len: usize,
) -> usize {
    if out_rgba.is_null() || out_len == 0 {
        return 0;
    }
    with_active_frontend(|core| {
        let frame = core.gfx.last_frame();
        if frame.width == 0 {
            return 0;
        }
        let rgba = &frame.rgba;
        let count = rgba.len().min(out_len);
        unsafe { ptr::copy_nonoverlapping(rgba.as_ptr(), out_rgba, count) };
        count
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_system_info(
    frontend: *const RfFrontend,
    out: *mut RfSystemInfo,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_ref() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    let sys_info = with_active_frontend(|core| core.system_info().cloned());
    if sys_info.is_none() {
        return false;
    }
    *out = RfSystemInfo {
        library_name: frontend.info_name.as_ptr(),
        library_version: frontend.info_version.as_ptr(),
        valid_extensions: frontend.info_extensions.as_ptr(),
        need_fullpath: sys_info.as_ref().map(|i| i.need_fullpath).unwrap_or(false),
        block_extract: sys_info.as_ref().map(|i| i.block_extract).unwrap_or(false),
    };
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_next_event(
    _frontend: *mut RfFrontend,
    out: *mut RfEvent,
) -> bool {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    let event = with_active_frontend(|core| core.next_event());
    if let Some(event) = event {
        *out = match event {
            FrontendEvent::VideoFrame {
                width,
                height,
                pitch,
                ..
            } => RfEvent {
                kind: 1,
                a: width as u64,
                b: height as u64,
                c: pitch as u64,
            },
            FrontendEvent::AudioBatch { frames } => RfEvent {
                kind: 2,
                a: frames as u64,
                b: 0,
                c: 0,
            },
            FrontendEvent::AudioSample { left, right } => RfEvent {
                kind: 3,
                a: left as i16 as u64,
                b: right as i16 as u64,
                c: 0,
            },
            FrontendEvent::EnvironmentCommand { command, handled } => RfEvent {
                kind: 4,
                a: command as u64,
                b: handled as u64,
                c: 0,
            },
            FrontendEvent::InputPoll => RfEvent {
                kind: 5,
                a: 0,
                b: 0,
                c: 0,
            },
        };
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_last_error(frontend: *const RfFrontend) -> *const c_char {
    let Some(frontend) = (unsafe { frontend.as_ref() }) else {
        return ptr::null();
    };
    frontend.last_error.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_options_config_path(
    _frontend: *mut RfFrontend,
    path: *const c_char,
) -> bool {
    let Some(path_str) = ptr_to_str(path) else {
        return false;
    };
    with_active_frontend(|core| core.options.set_config_path(PathBuf::from(path_str)));
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_options_count(_frontend: *const RfFrontend) -> usize {
    with_active_frontend(|core| core.options.definitions().len())
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_get_option(
    frontend: *mut RfFrontend,
    index: usize,
    out: *mut RfCoreOption,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };

    with_active_frontend(|core| {
        let defs = core.options.definitions();
        let Some(def) = defs.get(index) else {
            return false;
        };

        let current_value = core
            .options
            .get_variable(&def.key)
            .cloned()
            .unwrap_or_else(|| def.default_value.clone());

        let key_c = CString::new(def.key.as_str()).unwrap_or_default();
        let desc_c = CString::new(def.desc.as_str()).unwrap_or_default();
        let info_c = CString::new(def.info.as_str()).unwrap_or_default();
        let value_c = CString::new(current_value.as_str()).unwrap_or_default();

        let mut values_c = Vec::new();
        for v in &def.values {
            let val_c = CString::new(v.value.as_str()).unwrap_or_default();
            let label_c = CString::new(v.label.as_str()).unwrap_or_default();
            values_c.push(RfCoreOptionValue {
                value: val_c.as_ptr(),
                label: label_c.as_ptr(),
            });
            frontend.cached_options.push(val_c);
            frontend.cached_options.push(label_c);
        }

        out.key = key_c.as_ptr();
        out.desc = desc_c.as_ptr();
        out.info = info_c.as_ptr();
        out.value = value_c.as_ptr();
        out.values = values_c.as_ptr();
        out.values_count = values_c.len();

        frontend.cached_options.push(key_c);
        frontend.cached_options.push(desc_c);
        frontend.cached_options.push(info_c);
        frontend.cached_options.push(value_c);
        frontend.cached_option_values.push(values_c);

        true
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_option(
    _frontend: *mut RfFrontend,
    key: *const c_char,
    value: *const c_char,
) -> bool {
    let Some(key_str) = ptr_to_str(key) else {
        return false;
    };
    let Some(value_str) = ptr_to_str(value) else {
        return false;
    };
    with_active_frontend(|core| core.options.set_variable(key_str, value_str));
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_clear_options_cache(frontend: *mut RfFrontend) {
    if let Some(frontend) = unsafe { frontend.as_mut() } {
        frontend.cached_options.clear();
        frontend.cached_option_values.clear();
    }
}

// Core Discovery API Impl
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_info_dir(_frontend: *mut RfFrontend, path: *const c_char) {
    if let Some(path_str) = ptr_to_str(path) {
        with_active_frontend(|core| {
            core.settings.set("libretro_info_path", &path_str);
            core.core_info.set_info_dir(PathBuf::from(path_str));
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_scan_cores(
    _frontend: *mut RfFrontend,
    cores_dir: *const c_char,
) {
    if let Some(path_str) = ptr_to_str(cores_dir) {
        with_active_frontend(|core| core.core_info.scan_directory(Path::new(&path_str)));
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_scan_configured_cores(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| {
        let dir = core.settings.libretro_directory();
        core.core_info.scan_directory(&dir);
    });
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_all_extensions(frontend: *mut RfFrontend) -> *const c_char {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return ptr::null();
    };
    with_active_frontend(|core| {
        let extensions = CString::new(core.core_info.all_extensions.join("|")).unwrap_or_default();
        let ptr = extensions.as_ptr();
        frontend.cached_strings.push(extensions);
        ptr
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_cores_count(_frontend: *const RfFrontend) -> usize {
    with_active_frontend(|core| core.core_info.cores.len())
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_get_core_info(
    frontend: *mut RfFrontend,
    index: usize,
    out: *mut RfCoreInfo,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };

    with_active_frontend(|core| {
        let Some(info) = core.core_info.cores.get(index) else {
            return false;
        };

        let path_c = CString::new(info.path.to_string_lossy().as_bytes()).unwrap_or_default();
        let name_c = CString::new(info.display_name.as_str()).unwrap_or_default();
        let sys_c = CString::new(info.system_name.as_str()).unwrap_or_default();
        let ext_c = CString::new(info.supported_extensions.join("|").as_str()).unwrap_or_default();

        out.path = path_c.as_ptr();
        out.display_name = name_c.as_ptr();
        out.system_name = sys_c.as_ptr();
        out.supported_extensions = ext_c.as_ptr();

        frontend.cached_strings.push(path_c);
        frontend.cached_strings.push(name_c);
        frontend.cached_strings.push(sys_c);
        frontend.cached_strings.push(ext_c);

        true
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_scan_games(
    _frontend: *mut RfFrontend,
    directory: *const c_char,
    extensions: *const c_char,
) {
    if let Some(dir_str) = ptr_to_str(directory) {
        if let Some(ext_str) = ptr_to_str(extensions) {
            let exts: Vec<String> = ext_str.split("|").map(|s| s.to_string()).collect();
            with_active_frontend(|core| core.scanner.scan_directory(Path::new(&dir_str), &exts));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_games_count(_frontend: *const RfFrontend) -> usize {
    with_active_frontend(|core| core.scanner.games.len())
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_get_game_info(
    frontend: *mut RfFrontend,
    index: usize,
    out: *mut RfGameEntry,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };

    with_active_frontend(|core| {
        let Some(info) = core.scanner.games.get(index) else {
            return false;
        };

        let path_c = CString::new(info.path.to_string_lossy().as_bytes()).unwrap_or_default();
        let label_c = CString::new(info.label.as_str()).unwrap_or_default();

        out.path = path_c.as_ptr();
        out.label = label_c.as_ptr();

        frontend.cached_strings.push(path_c);
        frontend.cached_strings.push(label_c);

        true
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_plan_content_launch(
    frontend: *mut RfFrontend,
    path: *const c_char,
    preferred_core: *const c_char,
    out: *mut RfLaunchPlan,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    let Some(path_str) = ptr_to_str(path) else {
        return false;
    };
    let preferred = ptr_to_str(preferred_core).map(PathBuf::from);

    with_active_frontend(|core| {
        if core.core_info.cores.is_empty() {
            let dir = core.settings.libretro_directory();
            core.core_info.scan_directory(&dir);
        }
        let content_path = PathBuf::from(path_str);
        let plan = core.launcher.plan_content_launch(
            &content_path,
            &core.core_info,
            &core.settings,
            preferred.as_deref(),
        );

        let content_path_c =
            CString::new(plan.content_path.to_string_lossy().as_bytes()).unwrap_or_default();
        let content_extension_c = CString::new(plan.content_extension.as_str()).unwrap_or_default();
        let selected_core_path_c = CString::new(
            plan.selected_core
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
        )
        .unwrap_or_default();
        let reason_c = CString::new(plan.reason.as_str()).unwrap_or_default();

        out.content_path = content_path_c.as_ptr();
        out.content_extension = content_extension_c.as_ptr();
        out.decision = match plan.decision {
            launch::LaunchDecisionKind::NoCore => 0,
            launch::LaunchDecisionKind::Selected => 1,
            launch::LaunchDecisionKind::NeedsCoreChoice => 2,
        };
        out.selected_core_path = selected_core_path_c.as_ptr();
        out.candidate_count = plan.candidates.len();
        out.reason = reason_c.as_ptr();

        frontend.cached_strings.push(content_path_c);
        frontend.cached_strings.push(content_extension_c);
        frontend.cached_strings.push(selected_core_path_c);
        frontend.cached_strings.push(reason_c);
        true
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_launch_candidate_count(_frontend: *const RfFrontend) -> usize {
    with_active_frontend(|core| {
        core.launcher
            .last_plan
            .as_ref()
            .map(|plan| plan.candidates.len())
            .unwrap_or(0)
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_get_launch_candidate(
    frontend: *mut RfFrontend,
    index: usize,
    out: *mut RfCoreInfo,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };

    with_active_frontend(|core| {
        let Some(plan) = core.launcher.last_plan.as_ref() else {
            return false;
        };
        let Some(info) = plan.candidates.get(index) else {
            return false;
        };
        let path_c = CString::new(info.path.to_string_lossy().as_bytes()).unwrap_or_default();
        let name_c = CString::new(info.display_name.as_str()).unwrap_or_default();
        let sys_c = CString::new(info.system_name.as_str()).unwrap_or_default();
        let ext_c = CString::new(info.supported_extensions.join("|").as_str()).unwrap_or_default();

        out.path = path_c.as_ptr();
        out.display_name = name_c.as_ptr();
        out.system_name = sys_c.as_ptr();
        out.supported_extensions = ext_c.as_ptr();

        frontend.cached_strings.push(path_c);
        frontend.cached_strings.push(name_c);
        frontend.cached_strings.push(sys_c);
        frontend.cached_strings.push(ext_c);
        true
    })
}

// Menu Engine API Impl
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_menu_current_list(
    frontend: *mut RfFrontend,
    out_list: *mut RfMenuList,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(out_list) = (unsafe { out_list.as_mut() }) else {
        return false;
    };

    with_active_frontend(|core| {
        if let Some(list) = core.menu.current() {
            let title_c = CString::new(list.title.as_str()).unwrap_or_default();
            out_list.title = title_c.as_ptr();
            out_list.entry_count = list.entries.len();
            frontend.cached_strings.push(title_c);
            true
        } else {
            false
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_menu_get_entry(
    frontend: *mut RfFrontend,
    index: usize,
    out_entry: *mut RfMenuEntry,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(out_entry) = (unsafe { out_entry.as_mut() }) else {
        return false;
    };

    with_active_frontend(|core| {
        if let Some(list) = core.menu.current() {
            if let Some(entry) = list.entries.get(index) {
                let label_c = CString::new(entry.label.as_str()).unwrap_or_default();
                let sublabel_c = CString::new(entry.sublabel.as_str()).unwrap_or_default();
                let value_c = CString::new(entry.value.as_str()).unwrap_or_default();

                out_entry.label = label_c.as_ptr();
                out_entry.sublabel = sublabel_c.as_ptr();
                out_entry.kind = match entry.kind {
                    menu::MenuEntryKind::Action => 0,
                    menu::MenuEntryKind::Submenu => 1,
                    menu::MenuEntryKind::Toggle => 2,
                    menu::MenuEntryKind::Setting => 3,
                };
                out_entry.value = value_c.as_ptr();
                out_entry.action_id = entry.action_id;

                frontend.cached_strings.push(label_c);
                frontend.cached_strings.push(sublabel_c);
                frontend.cached_strings.push(value_c);
                true
            } else {
                false
            }
        } else {
            false
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_menu_push_core_list(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| {
        let cores = core.core_info.cores.clone();
        core.menu.push_core_list(&cores);
    });
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_menu_push_settings(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| {
        core.menu.push_settings(&core.settings);
    });
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_menu_push_content_list(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| {
        core.menu.push_content_list(&core.scanner.games);
    });
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_menu_push_information(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| {
        let system_info = core.system_info().cloned();
        let game_info = core.game_info().cloned();
        let gfx_status = core.gfx.driver_status().clone();
        core.menu
            .push_information(system_info.as_ref(), game_info.as_ref(), &gfx_status);
    });
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_menu_activate(
    _frontend: *mut RfFrontend,
    action_id: u32,
) -> bool {
    with_active_frontend(|core| match action_id {
        1 => {
            let cores = core.core_info.cores.clone();
            core.menu.push_core_list(&cores);
            true
        }
        2 => {
            core.menu.push_content_list(&core.scanner.games);
            true
        }
        3 => {
            core.menu.push_status(
                "Online Updater",
                "Network updater is not implemented yet in the Rust menu engine.",
            );
            true
        }
        4 => {
            core.menu.push_settings(&core.settings);
            true
        }
        5 => {
            let system_info = core.system_info().cloned();
            let game_info = core.game_info().cloned();
            let gfx_status = core.gfx.driver_status().clone();
            core.menu
                .push_information(system_info.as_ref(), game_info.as_ref(), &gfx_status);
            true
        }
        13 => {
            core.menu.push_status(
                "Shaders",
                "Shader configuration will be handled by the Rust video menu.",
            );
            true
        }
        14 => {
            core.menu
                .push_status("Save States", "Save-state actions are not implemented yet.");
            true
        }
        260..=262 => {
            core.menu.push_skin_settings(&core.settings);
            true
        }
        _ => false,
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_menu_push_skin_settings(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| {
        core.menu.push_skin_settings(&core.settings);
    });
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_menu_pop(_frontend: *mut RfFrontend) -> bool {
    with_active_frontend(|core| core.menu.pop().is_some())
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_load_settings(
    _frontend: *mut RfFrontend,
    path: *const c_char,
) -> bool {
    let Some(path_str) = ptr_to_str(path) else {
        return false;
    };
    with_active_frontend(|core| {
        core.settings.load(Path::new(&path_str));
        core.configure_from_settings();
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_base_dir(
    _frontend: *mut RfFrontend,
    path: *const c_char,
) -> bool {
    let Some(path_str) = ptr_to_str(path) else {
        return false;
    };
    with_active_frontend(|core| {
        let base_dir = PathBuf::from(&path_str);
        core.settings.set_base_dir(&base_dir);
        core.settings.set("libretro_directory", &base_dir.join("Cores").to_string_lossy());
        core.settings.set("libretro_info_path", &base_dir.join("info").to_string_lossy());
        core.settings.set("core_options_path", &base_dir.join("retroarch-core-options.cfg").to_string_lossy());
        core.configure_from_settings();
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_save_settings(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| core.settings.save());
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_get_setting(
    frontend: *mut RfFrontend,
    key: *const c_char,
) -> *const c_char {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return ptr::null();
    };
    let Some(key_str) = ptr_to_str(key) else {
        return ptr::null();
    };
    with_active_frontend(|core| {
        let value = core.settings.get(&key_str).cloned().unwrap_or_default();
        let value_c = CString::new(value).unwrap_or_default();
        let ptr = value_c.as_ptr();
        frontend.cached_strings.push(value_c);
        ptr
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_setting(
    _frontend: *mut RfFrontend,
    key: *const c_char,
    value: *const c_char,
) -> bool {
    let Some(key_str) = ptr_to_str(key) else {
        return false;
    };
    let Some(value_str) = ptr_to_str(value) else {
        return false;
    };
    with_active_frontend(|core| {
        core.settings.set(&key_str, &value_str);
        core.configure_from_settings();
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_settings_count(_frontend: *const RfFrontend) -> usize {
    with_active_frontend(|core| core.settings.values.len())
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_get_setting_at(
    frontend: *mut RfFrontend,
    index: usize,
    out: *mut RfSettingEntry,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    with_active_frontend(|core| {
        let mut entries: Vec<_> = core.settings.values.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        let Some((key, value)) = entries.get(index) else {
            return false;
        };
        let key_c = CString::new(key.as_str()).unwrap_or_default();
        let value_c = CString::new(value.as_str()).unwrap_or_default();
        out.key = key_c.as_ptr();
        out.value = value_c.as_ptr();
        frontend.cached_strings.push(key_c);
        frontend.cached_strings.push(value_c);
        true
    })
}

fn ptr_to_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(ptr) }.to_str().ok()?.to_owned())
    }
}

fn set_error(frontend: &mut RfFrontend, message: &str) {
    frontend.last_error = CString::new(message).unwrap_or_else(|_| {
        CString::new("error contained an interior NUL").expect("static CString")
    });
}

fn cache_system_info(frontend: &mut RfFrontend, info: &SystemInfo) {
    frontend.info_name = CString::new(info.library_name.as_str()).unwrap_or_default();
    frontend.info_version = CString::new(info.library_version.as_str()).unwrap_or_default();
    frontend.info_extensions = CString::new(info.valid_extensions.join("|")).unwrap_or_default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        let frontend = FrontendCore::new();
        assert_eq!(frontend.state(), SessionState::Empty);
        assert!(frontend.system_info().is_none());
    }

    #[test]
    fn joypad_buttons_are_mutable() {
        let mut frontend = FrontendCore::new();
        assert_eq!(frontend.joypad_button(8), 0);
        frontend.set_joypad_button(8, true).unwrap();
        assert_eq!(frontend.joypad_button(8), 1);
        frontend.set_joypad_button(8, false).unwrap();
        assert_eq!(frontend.joypad_button(8), 0);
    }
}
