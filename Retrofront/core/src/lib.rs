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
    GfxBackendKind, GfxRuntime, HardwareRenderRequest, HostRenderHandles, OpenGlRenderCommand,
    PixelFormat, VulkanRenderCommand, RETRO_HW_FRAME_BUFFER_VALID,
};
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uint, c_void};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::{Mutex, OnceLock};

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
        let library = Library::open(path)?;
        Ok(Self {
            retro_set_environment: sym!(library, "retro_set_environment", RetroSetEnvironment),
            retro_set_video_refresh: sym!(library, "retro_set_video_refresh", RetroSetVideoRefresh),
            retro_set_audio_sample: sym!(library, "retro_set_audio_sample", RetroSetAudioSample),
            retro_set_audio_sample_batch: sym!(
                library,
                "retro_set_audio_sample_batch",
                RetroSetAudioSampleBatch
            ),
            retro_set_input_poll: sym!(library, "retro_set_input_poll", RetroSetInputPoll),
            retro_set_input_state: sym!(library, "retro_set_input_state", RetroSetInputState),
            retro_init: sym!(library, "retro_init", RetroInit),
            retro_deinit: sym!(library, "retro_deinit", RetroDeinit),
            retro_api_version: sym!(library, "retro_api_version", RetroApiVersion),
            retro_get_system_info: sym!(library, "retro_get_system_info", RetroGetSystemInfo),
            retro_get_system_av_info: sym!(
                library,
                "retro_get_system_av_info",
                RetroGetSystemAvInfo
            ),
            retro_load_game: sym!(library, "retro_load_game", RetroLoadGame),
            retro_unload_game: sym!(library, "retro_unload_game", RetroUnloadGame),
            retro_run: sym!(library, "retro_run", RetroRun),
            _library: library,
        })
    }
}

#[derive(Default)]
pub struct FrontendCore {
    api: Option<CoreApi>,
    system_info: Option<SystemInfo>,
    game: Option<GameInfo>,
    events: VecDeque<FrontendEvent>,
    gfx: GfxRuntime,
    last_error: Option<String>,
}

impl Drop for FrontendCore {
    fn drop(&mut self) {
        self.unload_game();
        if let Some(api) = self.api.take() {
            unsafe { (api.retro_deinit)() };
        }
    }
}

impl FrontendCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> SessionState {
        match (&self.api, &self.game) {
            (None, _) => SessionState::Empty,
            (Some(_), None) => SessionState::CoreLoaded,
            (Some(_), Some(_)) => SessionState::GameLoaded,
        }
    }

    pub fn system_info(&self) -> Option<&SystemInfo> {
        self.system_info.as_ref()
    }
    pub fn game_info(&self) -> Option<&GameInfo> {
        self.game.as_ref()
    }
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
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

    pub fn load_core(&mut self, path: impl AsRef<Path>) -> Result<SystemInfo, String> {
        self.drop_current_core();
        let api = CoreApi::load(path).map_err(|e| self.record_error(e))?;

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
            return Err(self.record_error(format!(
                "unsupported libretro API version {}; expected {}",
                version,
                libretro::RETRO_API_VERSION
            )));
        }

        set_active_frontend(self as *mut FrontendCore);
        unsafe { (api.retro_init)() };
        clear_active_frontend();
        let info = read_system_info(&api);
        self.system_info = Some(info.clone());
        self.api = Some(api);
        self.last_error = None;
        Ok(info)
    }

    pub fn load_game(&mut self, path: impl AsRef<Path>, meta: Option<&str>) -> Result<(), String> {
        if self.api.is_none() {
            return Err(self.record_error("load a core before loading a game"));
        }
        self.unload_game();
        let retro_load_game = self
            .api
            .as_ref()
            .expect("core presence checked before unloading game")
            .retro_load_game;
        let path = path.as_ref().to_path_buf();
        let c_path = CString::new(path.to_string_lossy().as_bytes()).map_err(|_| {
            self.record_error(format!(
                "game path contains an interior NUL: {}",
                path.display()
            ))
        })?;
        let c_meta = match meta {
            Some(value) => Some(
                CString::new(value)
                    .map_err(|_| self.record_error("game metadata contains an interior NUL"))?,
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
                self.record_error(format!("libretro core rejected game {}", path.display()))
            );
        }
        self.game = Some(GameInfo {
            path,
            meta: meta.map(ToOwned::to_owned),
        });
        self.refresh_system_av_info();
        self.last_error = None;
        Ok(())
    }

    pub fn run_frame(&mut self) -> Result<(), String> {
        let retro_run = self
            .api
            .as_ref()
            .ok_or_else(|| self.record_error("load a core before running a frame"))?
            .retro_run;
        if self.game.is_none() {
            return Err(self.record_error("load a game before running a frame"));
        }
        set_active_frontend(self as *mut FrontendCore);
        unsafe { (retro_run)() };
        clear_active_frontend();
        self.last_error = None;
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

    fn record_error(&self, message: impl Into<String>) -> String {
        message.into()
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

fn set_active_frontend(frontend: *mut FrontendCore) {
    *active_frontend()
        .lock()
        .expect("active frontend mutex poisoned") = Some(frontend as usize);
}

fn clear_active_frontend() {
    *active_frontend()
        .lock()
        .expect("active frontend mutex poisoned") = None;
}

fn with_active_frontend<R>(f: impl FnOnce(&mut FrontendCore) -> R) -> Option<R> {
    let ptr = *active_frontend()
        .lock()
        .expect("active frontend mutex poisoned");
    ptr.map(|ptr| {
        let frontend = ptr as *mut FrontendCore;
        unsafe { f(&mut *frontend) }
    })
}

fn push_event(event: FrontendEvent) {
    let _ = with_active_frontend(|frontend| frontend.events.push_back(event));
}

unsafe extern "C" fn environment_callback(cmd: c_uint, data: *mut c_void) -> bool {
    let handled = match cmd {
        libretro::RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => unsafe {
            let Some(raw) = data.cast::<c_uint>().as_ref() else {
                push_event(FrontendEvent::EnvironmentCommand {
                    command: cmd,
                    handled: false,
                });
                return false;
            };
            let Some(format) = PixelFormat::from_libretro(*raw) else {
                push_event(FrontendEvent::EnvironmentCommand {
                    command: cmd,
                    handled: false,
                });
                return false;
            };
            with_active_frontend(|frontend| frontend.gfx.set_pixel_format(format));
            true
        },
        libretro::RETRO_ENVIRONMENT_GET_CAN_DUPE => unsafe {
            if let Some(can_dupe) = data.cast::<bool>().as_mut() {
                *can_dupe = true;
                true
            } else {
                false
            }
        },
        libretro::RETRO_ENVIRONMENT_SET_HW_RENDER => unsafe {
            let Some(raw) = data.cast::<libretro::retro_hw_render_callback>().as_mut() else {
                return false;
            };
            let request = HardwareRenderRequest::from_libretro(raw);
            with_active_frontend(|frontend| {
                frontend.gfx.set_hardware_render_request(request);
                frontend.gfx.patch_hw_render_callback(raw);
            });
            true
        },
        libretro::RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER => unsafe {
            let Some(out) = data.cast::<c_uint>().as_mut() else {
                return false;
            };
            let preferred = with_active_frontend(|frontend| frontend.gfx.backend())
                .unwrap_or(GfxBackendKind::Software);
            *out = match preferred {
                GfxBackendKind::Software => libretro::retro_hw_context_type_RETRO_HW_CONTEXT_NONE,
                GfxBackendKind::OpenGl => {
                    libretro::retro_hw_context_type_RETRO_HW_CONTEXT_OPENGLES_VERSION
                }
                GfxBackendKind::Vulkan => libretro::retro_hw_context_type_RETRO_HW_CONTEXT_VULKAN,
            };
            true
        },
        libretro::RETRO_ENVIRONMENT_SET_GEOMETRY
        | libretro::RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO
        | libretro::RETRO_ENVIRONMENT_GET_LOG_INTERFACE
        | libretro::RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME => true,
        _ => false,
    };
    push_event(FrontendEvent::EnvironmentCommand {
        command: cmd,
        handled,
    });
    handled
}

unsafe extern "C" fn video_refresh_callback(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    let _ = with_active_frontend(|frontend| {
        let pixel_format = frontend.gfx.pixel_format().code();
        let frame_number = match if data == RETRO_HW_FRAME_BUFFER_VALID {
            frontend
                .gfx
                .ingest_hardware_frame(width, height)
                .map(|_| frontend.gfx.last_frame())
        } else {
            frontend
                .gfx
                .ingest_software_frame(data, width, height, pitch)
        } {
            Ok(frame) => {
                let _ = (frame.width, frame.height);
                frontend.gfx.frame_counter()
            }
            Err(error) => {
                frontend.last_error = Some(error);
                frontend.gfx.frame_counter()
            }
        };
        frontend.events.push_back(FrontendEvent::VideoFrame {
            width,
            height,
            pitch,
            pixel_format,
            frame_number,
        });
    });
}

unsafe extern "C" fn audio_sample_callback(left: i16, right: i16) {
    push_event(FrontendEvent::AudioSample { left, right });
}

unsafe extern "C" fn audio_sample_batch_callback(_data: *const i16, frames: usize) -> usize {
    push_event(FrontendEvent::AudioBatch { frames });
    frames
}

unsafe extern "C" fn input_poll_callback() {
    push_event(FrontendEvent::InputPoll);
}

unsafe extern "C" fn input_state_callback(
    _port: c_uint,
    _device: c_uint,
    _index: c_uint,
    _id: c_uint,
) -> i16 {
    0
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
    pub kind: c_uint,
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
pub struct RfGfxHostHandles {
    pub native_view: u64,
    pub gl_context: u64,
    pub gl_framebuffer: usize,
    pub vulkan_instance: u64,
    pub vulkan_device: u64,
    pub vulkan_queue: u64,
    pub vulkan_command_buffer: u64,
    pub vulkan_image: u64,
    pub opengl_render: Option<
        unsafe extern "C" fn(*const OpenGlRenderCommand, *const u8, usize, *mut c_void) -> bool,
    >,
    pub vulkan_render: Option<
        unsafe extern "C" fn(*const VulkanRenderCommand, *const u8, usize, *mut c_void) -> bool,
    >,
    pub get_proc_address:
        Option<unsafe extern "C" fn(*const c_char, *mut c_void) -> libretro::retro_proc_address_t>,
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
pub struct RfFrontend {
    inner: FrontendCore,
    info_name: CString,
    info_version: CString,
    info_extensions: CString,
    last_error: CString,
}

#[no_mangle]
pub extern "C" fn rf_frontend_create() -> *mut RfFrontend {
    Box::into_raw(Box::new(RfFrontend {
        inner: FrontendCore::new(),
        info_name: CString::default(),
        info_version: CString::default(),
        info_extensions: CString::default(),
        last_error: CString::default(),
    }))
}

/// Destroys a frontend allocated by [`rf_frontend_create`].
///
/// # Safety
///
/// `frontend` must be either null or a pointer returned by `rf_frontend_create`
/// that has not already been destroyed.
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_destroy(frontend: *mut RfFrontend) {
    if !frontend.is_null() {
        drop(unsafe { Box::from_raw(frontend) });
    }
}

/// Returns the current frontend state code.
///
/// # Safety
///
/// `frontend` must be null or a valid pointer returned by `rf_frontend_create`.
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_state(frontend: *const RfFrontend) -> c_uint {
    let Some(frontend) = (unsafe { frontend.as_ref() }) else {
        return 0;
    };
    match frontend.inner.state() {
        SessionState::Empty => 0,
        SessionState::CoreLoaded => 1,
        SessionState::GameLoaded => 2,
    }
}

/// Loads and initializes a libretro core from `path`.
///
/// # Safety
///
/// `frontend` must be a valid pointer returned by `rf_frontend_create`; `path`
/// must point to a valid null-terminated UTF-8 string for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_load_core(
    frontend: *mut RfFrontend,
    path: *const c_char,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(path) = ptr_to_str(path) else {
        set_error(frontend, "core path is null or invalid UTF-8");
        return false;
    };
    match frontend.inner.load_core(path) {
        Ok(info) => {
            cache_system_info(frontend, &info);
            true
        }
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

/// Loads a game into the active libretro core.
///
/// # Safety
///
/// `frontend` must be a valid pointer returned by `rf_frontend_create`; `path`
/// must be a valid null-terminated UTF-8 string, and `meta` must be null or a
/// valid null-terminated UTF-8 string for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_load_game(
    frontend: *mut RfFrontend,
    path: *const c_char,
    meta: *const c_char,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(path) = ptr_to_str(path) else {
        set_error(frontend, "game path is null or invalid UTF-8");
        return false;
    };
    let meta = if meta.is_null() {
        None
    } else {
        ptr_to_str(meta)
    };
    match frontend.inner.load_game(path, meta.as_deref()) {
        Ok(()) => true,
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

/// Runs one frame on the loaded game.
///
/// # Safety
///
/// `frontend` must be a valid pointer returned by `rf_frontend_create`.
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_run_frame(frontend: *mut RfFrontend) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    match frontend.inner.run_frame() {
        Ok(()) => true,
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

/// Unloads the current game if one is loaded.
///
/// # Safety
///
/// `frontend` must be null or a valid pointer returned by `rf_frontend_create`.
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_unload_game(frontend: *mut RfFrontend) {
    if let Some(frontend) = unsafe { frontend.as_mut() } {
        frontend.inner.unload_game();
    }
}

/// Selects the shared Rust gfx backend: 0 software, 1 OpenGL/OpenGL ES, 2 Vulkan/MoltenVK.
///
/// # Safety
///
/// `frontend` must be a valid pointer returned by `rf_frontend_create`.
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_gfx_backend(
    frontend: *mut RfFrontend,
    backend: c_uint,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(backend) = GfxBackendKind::from_code(backend) else {
        set_error(frontend, "unknown gfx backend");
        return false;
    };
    frontend.inner.set_gfx_backend(backend);
    true
}

/// Supplies native OpenGL ES or Vulkan/MoltenVK handles to the Rust gfx driver.
///
/// # Safety
///
/// `frontend` must be a valid pointer returned by `rf_frontend_create`; `handles`
/// must be readable. Handles are opaque and remain owned by the native host.
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_gfx_host_handles(
    frontend: *mut RfFrontend,
    handles: *const RfGfxHostHandles,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(handles) = (unsafe { handles.as_ref() }) else {
        set_error(frontend, "gfx host handles are null");
        return false;
    };
    frontend.inner.gfx.set_host_handles(HostRenderHandles {
        native_view: handles.native_view,
        gl_context: handles.gl_context,
        gl_framebuffer: handles.gl_framebuffer,
        vulkan_instance: handles.vulkan_instance,
        vulkan_device: handles.vulkan_device,
        vulkan_queue: handles.vulkan_queue,
        vulkan_command_buffer: handles.vulkan_command_buffer,
        vulkan_image: handles.vulkan_image,
        opengl_render: handles.opengl_render,
        vulkan_render: handles.vulkan_render,
        get_proc_address: handles.get_proc_address,
        user_data: handles.user_data,
    });
    true
}

/// Returns the active gfx driver status, including hardware readiness.
///
/// # Safety
///
/// `frontend` must be valid and `out` must be writable.
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
    *out = RfGfxDriverInfo {
        backend: status.backend as u32,
        frame_number: status.frame_counter,
        hardware_ready: status.hardware_ready,
        rendered: status
            .last_present
            .as_ref()
            .is_some_and(|present| present.rendered),
    };
    true
}

/// Returns metadata for the latest core-provided video frame copied by Rust gfx.
///
/// # Safety
///
/// `frontend` must be a valid pointer returned by `rf_frontend_create`; `out`
/// must be writable.
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
    let frame = frontend.inner.gfx().last_frame();
    if frame.rgba.is_empty() || frame.width == 0 || frame.height == 0 {
        return false;
    }
    *out = RfVideoFrameInfo {
        width: frame.width,
        height: frame.height,
        pitch: frame.pitch as u64,
        rgba_len: frame.rgba.len() as u64,
        pixel_format: frame.source_format.code(),
        frame_number: frontend.inner.gfx().frame_counter(),
    };
    true
}

/// Copies the latest RGBA8888 frame into `out_rgba` and returns bytes copied.
///
/// # Safety
///
/// `frontend` must be valid. `out_rgba` must be writable for `out_len` bytes.
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
    let rgba = &frontend.inner.gfx().last_frame().rgba;
    let count = rgba.len().min(out_len);
    unsafe { ptr::copy_nonoverlapping(rgba.as_ptr(), out_rgba, count) };
    count
}

/// Copies the built-in OpenGL ES shaders used to display RGBA frames.
///
/// # Safety
///
/// Output pointers may be null; returned strings have static lifetime.
#[no_mangle]
pub unsafe extern "C" fn rf_frontend_opengl_shader_sources(
    vertex_out: *mut *const c_char,
    fragment_out: *mut *const c_char,
) {
    static VERTEX: &[u8] = concat!(
        "#version 300 es\n",
        "precision mediump float;\n",
        "layout(location = 0) in vec2 a_position;\n",
        "layout(location = 1) in vec2 a_texcoord;\n",
        "out vec2 v_texcoord;\n",
        "void main() {\n",
        "    v_texcoord = a_texcoord;\n",
        "    gl_Position = vec4(a_position, 0.0, 1.0);\n",
        "}\n",
        "\0"
    )
    .as_bytes();
    static FRAGMENT: &[u8] = concat!(
        "#version 300 es\n",
        "precision mediump float;\n",
        "in vec2 v_texcoord;\n",
        "uniform sampler2D u_frame;\n",
        "out vec4 color;\n",
        "void main() {\n",
        "    color = texture(u_frame, v_texcoord);\n",
        "}\n",
        "\0"
    )
    .as_bytes();
    if let Some(out) = unsafe { vertex_out.as_mut() } {
        *out = VERTEX.as_ptr().cast();
    }
    if let Some(out) = unsafe { fragment_out.as_mut() } {
        *out = FRAGMENT.as_ptr().cast();
    }
}

/// Copies cached system information into `out`.
///
/// # Safety
///
/// `frontend` must be a valid pointer returned by `rf_frontend_create`; `out`
/// must be a valid writable pointer. Returned string pointers are owned by the
/// frontend and remain valid until the next core load or destruction.
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

/// Pops the next captured frontend event into `out`.
///
/// # Safety
///
/// `frontend` must be a valid pointer returned by `rf_frontend_create`; `out`
/// must be a valid writable pointer.
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
            a: left as i64 as u64,
            b: right as i64 as u64,
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

/// Returns the last error string owned by the frontend.
///
/// # Safety
///
/// `frontend` must be null or a valid pointer returned by `rf_frontend_create`.
/// The returned pointer remains valid until the next mutating call or destruction.
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
    frontend.last_error = CString::default();
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
    fn splits_extensions() {
        assert_eq!(
            "nes|fds|unif"
                .split('|')
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<String>>(),
            vec!["nes", "fds", "unif"]
        );
    }
}
