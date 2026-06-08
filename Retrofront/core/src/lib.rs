//! Retrofront Rust management core.
//!
//! This crate is **not** an emulator core. It is the frontend's portable
//! management layer: it loads libretro cores, owns session state, stores ROM
//! metadata, and exposes a stable C ABI for Swift UI code on iOS and Linux.

mod dylib;
pub mod gfx;
pub mod libretro;

use dylib::Library;
use gfx::{
    GfxBackendKind, GfxRuntime, HardwareRenderRequest,
    HostRenderHandles, PixelFormat,
};
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uint, c_void};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::{Mutex, OnceLock};

pub const RETRO_HW_FRAME_BUFFER_VALID: *const u8 = usize::MAX as *const u8;

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
        None // last_error is managed in RfFrontend
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

    pub fn set_joypad_button(&mut self, button_id: u32, pressed: bool) -> Result<(), String> {
        let Some(slot) = self.joypad_buttons.get_mut(button_id as usize) else {
            return Err(format!("unknown joypad button id {button_id}"));
        };
        *slot = if pressed { 1 } else { 0 };
        Ok(())
    }

    pub fn joypad_button(&self, button_id: u32) -> i16 {
        self.joypad_buttons
            .get(button_id as usize)
            .copied()
            .unwrap_or(0)
    }

    pub fn load_core(&mut self, path: impl AsRef<Path>) -> Result<SystemInfo, String> {
        self.drop_current_core();
        let api = CoreApi::load(path)?;

        unsafe {
            (api.retro_set_environment)(Some(environment_callback));
            (api.retro_set_video_refresh)(Some(video_refresh_callback));
            (api.retro_set_audio_sample)(Some(audio_sample_callback));
            (api.retro_set_audio_sample_batch)(Some(audio_sample_batch_callback));
            (api.retro_set_input_poll)(Some(input_poll_callback));
            (api.retro_set_input_state)(Some(input_state_callback));
        }

        let version = unsafe { (api.retro_api_version)() };
        if version != libretro::RETRO_API_VERSION {
            return Err(format!(
                "unsupported libretro API version {}; expected {}",
                version,
                libretro::RETRO_API_VERSION
            ));
        }

        set_active_frontend(self as *mut FrontendCore);
        unsafe { (api.retro_init)() };
        clear_active_frontend();
        let info = read_system_info(&api);
        self.system_info = Some(info.clone());
        self.api = Some(api);
        Ok(info)
    }

    pub fn load_game(&mut self, path: impl AsRef<Path>, meta: Option<&str>) -> Result<(), String> {
        if self.api.is_none() {
            return Err("load a core before loading a game".to_string());
        }
        self.unload_game();
        let retro_load_game = self
            .api
            .as_ref()
            .expect("core presence checked before unloading game")
            .retro_load_game;
        let path = path.as_ref().to_path_buf();
        let c_path = CString::new(path.to_string_lossy().as_bytes()).map_err(|_| {
            format!(
                "game path contains an interior NUL: {}",
                path.display()
            )
        })?;
        let c_meta = match meta {
            Some(value) => Some(
                CString::new(value)
                    .map_err(|_| "game metadata contains an interior NUL")?,
            ),
            None => None,
        };
        let raw = libretro::retro_game_info {
            path: c_path.as_ptr(),
            data: ptr::null(),
            size: 0,
            meta: c_meta.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
        };
        set_active_frontend(self as *mut FrontendCore);
        let loaded = unsafe { (retro_load_game)(&raw) };
        clear_active_frontend();
        if !loaded {
            return Err(
                format!("libretro core rejected game {}", path.display())
            );
        }
        self.game = Some(GameInfo {
            path,
            meta: meta.map(ToOwned::to_owned),
        });
        self.refresh_system_av_info();
        Ok(())
    }

    pub fn run_frame(&mut self) -> Result<(), String> {
        let retro_run = self
            .api
            .as_ref()
            .ok_or_else(|| "load a core before running a frame".to_string())?
            .retro_run;
        if self.game.is_none() {
            return Err("load a game before running a frame".to_string());
        }
        set_active_frontend(self as *mut FrontendCore);
        unsafe { (retro_run)() };
        clear_active_frontend();
        Ok(())
    }

    pub fn unload_game(&mut self) {
        if self.game.take().is_some() {
            if let Some(api) = self.api.as_ref() {
                unsafe { (api.retro_unload_game)() };
            }
        }
    }

    fn drop_current_core(&mut self) {
        self.unload_game();
        if let Some(api) = self.api.take() {
            unsafe { (api.retro_deinit)() };
        }
        self.system_info = None;
        self.events.clear();
        self.gfx = GfxRuntime::new();
    }

    fn refresh_system_av_info(&mut self) {
        let Some(api) = self.api.as_ref() else { return };
        let mut av = unsafe { std::mem::zeroed::<libretro::retro_system_av_info>() };
        unsafe { (api.retro_get_system_av_info)(&mut av) };
        self.events.push_back(FrontendEvent::EnvironmentCommand {
            command: libretro::RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO,
            handled: true,
        });
    }
}

fn read_system_info(api: &CoreApi) -> SystemInfo {
    let mut raw = libretro::retro_system_info {
        library_name: ptr::null(),
        library_version: ptr::null(),
        valid_extensions: ptr::null(),
        need_fullpath: false,
        block_extract: false,
    };
    unsafe { (api.retro_get_system_info)(&mut raw) };
    SystemInfo {
        library_name: c_string(raw.library_name),
        library_version: c_string(raw.library_version),
        valid_extensions: c_string(raw.valid_extensions)
            .split('|')
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        need_fullpath: raw.need_fullpath,
        block_extract: raw.block_extract,
    }
}

fn c_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}

fn active_frontend() -> &'static Mutex<Option<usize>> {
    static ACTIVE: OnceLock<Mutex<Option<usize>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

fn set_active_frontend(ptr: *mut FrontendCore) {
    let mut active = active_frontend().lock().unwrap();
    *active = Some(ptr as usize);
}

fn clear_active_frontend() {
    let mut active = active_frontend().lock().unwrap();
    *active = None;
}

pub fn with_active_frontend<R>(f: impl FnOnce(&mut FrontendCore) -> R) -> Option<R> {
    let active = active_frontend().lock().unwrap();
    active.map(|ptr| unsafe { f(&mut *(ptr as *mut FrontendCore)) })
}

unsafe extern "C" fn environment_callback(command: c_uint, data: *mut c_void) -> bool {
    with_active_frontend(|frontend| match command {
        libretro::RETRO_ENVIRONMENT_GET_CAN_DUPE => {
            if !data.is_null() {
                unsafe { *(data.cast::<bool>()) = true };
            }
            true
        }
        libretro::RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
            if data.is_null() {
                return false;
            }
            let format = unsafe { *(data.cast::<u32>()) };
            if let Some(pixel_format) = PixelFormat::from_libretro(format) {
                frontend.gfx.set_pixel_format(pixel_format);
                true
            } else {
                false
            }
        }
        libretro::RETRO_ENVIRONMENT_SET_HW_RENDER => {
            if data.is_null() {
                return false;
            }
            let raw = unsafe { &mut *(data.cast::<libretro::retro_hw_render_callback>()) };
            let request = HardwareRenderRequest::from_libretro(raw);
            frontend.gfx.set_hardware_render_request(request);
            frontend.gfx.patch_hw_render_callback(raw);
            true
        }
        _ => {
            frontend.events.push_back(FrontendEvent::EnvironmentCommand {
                command,
                handled: false,
            });
            false
        }
    })
    .unwrap_or(false)
}

unsafe extern "C" fn video_refresh_callback(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    with_active_frontend(|frontend| {
        let frame_number = if data == RETRO_HW_FRAME_BUFFER_VALID as *const c_void {
            frontend.gfx.ingest_hardware_frame(width, height).ok();
            frontend.gfx.frame_counter()
        } else if !data.is_null() {
            frontend.gfx.ingest_software_frame(data.cast(), width, height, pitch).ok();
            frontend.gfx.frame_counter()
        } else {
            0
        };

        frontend.events.push_back(FrontendEvent::VideoFrame {
            width,
            height,
            pitch,
            pixel_format: 0, // TODO
            frame_number,
        });
    });
}

unsafe extern "C" fn audio_sample_callback(left: i16, right: i16) {
    with_active_frontend(|frontend| {
        frontend
            .events
            .push_back(FrontendEvent::AudioSample { left, right });
    });
}

unsafe extern "C" fn audio_sample_batch_callback(_data: *const i16, frames: usize) -> usize {
    with_active_frontend(|frontend| {
        frontend
            .events
            .push_back(FrontendEvent::AudioBatch { frames });
        frames
    })
    .unwrap_or(0)
}

unsafe extern "C" fn input_poll_callback() {
    with_active_frontend(|frontend| {
        frontend.events.push_back(FrontendEvent::InputPoll);
    });
}

unsafe extern "C" fn input_state_callback(
    port: c_uint,
    device: c_uint,
    index: c_uint,
    id: c_uint,
) -> i16 {
    with_active_frontend(|frontend| {
        if port == 0 && device == libretro::RETRO_DEVICE_JOYPAD && index == 0 {
            frontend.joypad_button(id)
        } else {
            0
        }
    })
    .unwrap_or(0)
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
    pub render_callback: Option<gfx::hardware::BgfxRenderCallback>,
    pub get_proc_address: Option<gfx::hardware::GetProcAddressCallback>,
    pub user_data: *mut c_void,
}

#[repr(C)]
pub struct RfGfxDriverInfo {
    pub backend: u32,
    pub frame_number: u64,
    pub hardware_ready: bool,
    pub rendered: bool,
}

pub struct RfFrontend {
    inner: FrontendCore,
    info_name: CString,
    info_version: CString,
    info_extensions: CString,
    last_error: CString,
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_create() -> *mut RfFrontend {
    Box::into_raw(Box::new(RfFrontend {
        inner: FrontendCore::new(),
        info_name: CString::default(),
        info_version: CString::default(),
        info_extensions: CString::default(),
        last_error: CString::default(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_destroy(frontend: *mut RfFrontend) {
    if !frontend.is_null() {
        unsafe { drop(Box::from_raw(frontend)) };
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_state(frontend: *const RfFrontend) -> u32 {
    let Some(frontend) = (unsafe { frontend.as_ref() }) else {
        return 0;
    };
    match frontend.inner.state() {
        SessionState::Empty => 0,
        SessionState::CoreLoaded => 1,
        SessionState::GameLoaded => 2,
    }
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
    match frontend.inner.load_core(path_str) {
        Ok(info) => {
            cache_system_info(frontend, &info);
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
    match frontend.inner.load_game(path_str, meta_str.as_deref()) {
        Ok(()) => {
            frontend.last_error = CString::default();
            true
        },
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
    match frontend.inner.run_frame() {
        Ok(()) => {
            frontend.last_error = CString::default();
            true
        },
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_unload_game(frontend: *mut RfFrontend) {
    if let Some(frontend) = unsafe { frontend.as_mut() } {
        frontend.inner.unload_game();
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_gfx_backend(
    frontend: *mut RfFrontend,
    backend_code: u32,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    if let Some(backend) = GfxBackendKind::from_code(backend_code) {
        frontend.inner.set_gfx_backend(backend);
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_gfx_video_config(
    frontend: *mut RfFrontend,
    config: *const RfGfxVideoConfig,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(config) = (unsafe { config.as_ref() }) else {
        return false;
    };
    frontend.inner.gfx.set_video_config(gfx::config::GfxVideoConfig {
        base_width: config.base_width,
        base_height: config.base_height,
        max_width: config.max_width,
        max_height: config.max_height,
        aspect_ratio: config.aspect_ratio,
        output_width: config.output_width,
        output_height: config.output_height,
        scale_mode: match config.scale_mode {
            1 => gfx::config::GfxScaleMode::KeepAspect,
            2 => gfx::config::GfxScaleMode::Integer,
            _ => gfx::config::GfxScaleMode::Stretch,
        },
        filter_mode: match config.filter_mode {
            1 => gfx::config::GfxFilterMode::Linear,
            _ => gfx::config::GfxFilterMode::Nearest,
        },
        rotation_quarters: config.rotation_quarters,
        vsync: config.vsync,
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_gfx_host_handles(
    frontend: *mut RfFrontend,
    handles: *const RfGfxHostHandles,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(handles) = (unsafe { handles.as_ref() }) else {
        return false;
    };
    frontend.inner.gfx.set_host_handles(HostRenderHandles {
        native_view: handles.native_view,
        context: handles.context,
        framebuffer: handles.framebuffer,
        render_callback: handles.render_callback,
        get_proc_address: handles.get_proc_address,
        user_data: handles.user_data,
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_joypad_button(
    frontend: *mut RfFrontend,
    button_id: c_uint,
    pressed: bool,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    match frontend.inner.set_joypad_button(button_id, pressed) {
        Ok(()) => {
            frontend.last_error = CString::default();
            true
        },
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_gfx_driver_info(
    frontend: *const RfFrontend,
    out: *mut RfGfxDriverInfo,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_ref() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    let status = frontend.inner.gfx.driver_status();
    let last_present = status.last_present.as_ref();
    *out = RfGfxDriverInfo {
        backend: last_present.map_or(0, |p| p.backend as u32),
        frame_number: status.frame_counter,
        hardware_ready: status.hardware_ready,
        rendered: last_present.is_some_and(|p| p.rendered),
    };
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_video_frame_info(
    frontend: *const RfFrontend,
    out: *mut RfVideoFrameInfo,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_ref() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    let frame = frontend.inner.gfx.last_frame();
    if frame.width == 0 {
        return false;
    }
    *out = RfVideoFrameInfo {
        width: frame.width,
        height: frame.height,
        pitch: frame.pitch as u64,
        rgba_len: frame.rgba.len() as u64,
        pixel_format: frame.source_format.code(),
        frame_number: frontend.inner.gfx.frame_counter(),
    };
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_copy_video_frame_rgba(
    frontend: *const RfFrontend,
    out_rgba: *mut u8,
    out_len: usize,
) -> usize {
    let Some(frontend) = (unsafe { frontend.as_ref() }) else {
        return 0;
    };
    if out_rgba.is_null() || out_len == 0 {
        return 0;
    }
    let frame = frontend.inner.gfx.last_frame();
    if frame.width == 0 {
        return 0;
    }
    let rgba = &frame.rgba;
    let count = rgba.len().min(out_len);
    unsafe { ptr::copy_nonoverlapping(rgba.as_ptr(), out_rgba, count) };
    count
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
    if frontend.inner.system_info().is_none() {
        return false;
    }
    *out = RfSystemInfo {
        library_name: frontend.info_name.as_ptr(),
        library_version: frontend.info_version.as_ptr(),
        valid_extensions: frontend.info_extensions.as_ptr(),
        need_fullpath: frontend
            .inner
            .system_info()
            .map(|i| i.need_fullpath)
            .unwrap_or(false),
        block_extract: frontend
            .inner
            .system_info()
            .map(|i| i.block_extract)
            .unwrap_or(false),
    };
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_next_event(
    frontend: *mut RfFrontend,
    out: *mut RfEvent,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    let Some(event) = frontend.inner.next_event() else {
        return false;
    };
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
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_last_error(frontend: *const RfFrontend) -> *const c_char {
    let Some(frontend) = (unsafe { frontend.as_ref() }) else {
        return ptr::null();
    };
    frontend.last_error.as_ptr()
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
