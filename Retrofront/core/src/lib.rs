//! Retrofront Rust management core.
//!
//! This crate is **not** an emulator core. It is the frontend's portable
//! management layer: it loads libretro cores, owns session state, stores ROM
//! metadata, and exposes a stable C ABI for Swift UI code on iOS and Linux.

mod dylib;
pub mod libretro;

use dylib::Library;
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

        unsafe { (api.retro_init)() };
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
        let api = self
            .api
            .as_ref()
            .expect("core presence checked before unloading game");
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
        let loaded = unsafe { (api.retro_load_game)(&raw) };
        if !loaded {
            return Err(
                self.record_error(format!("libretro core rejected game {}", path.display()))
            );
        }
        self.game = Some(GameInfo {
            path,
            meta: meta.map(ToOwned::to_owned),
        });
        self.last_error = None;
        Ok(())
    }

    pub fn run_frame(&mut self) -> Result<(), String> {
        let api = self
            .api
            .as_ref()
            .ok_or_else(|| self.record_error("load a core before running a frame"))?;
        if self.game.is_none() {
            return Err(self.record_error("load a game before running a frame"));
        }
        set_active_events(&mut self.events as *mut VecDeque<FrontendEvent>);
        unsafe { (api.retro_run)() };
        clear_active_events();
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

fn active_events() -> &'static Mutex<Option<usize>> {
    static ACTIVE: OnceLock<Mutex<Option<usize>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

fn set_active_events(events: *mut VecDeque<FrontendEvent>) {
    *active_events()
        .lock()
        .expect("active events mutex poisoned") = Some(events as usize);
}

fn clear_active_events() {
    *active_events()
        .lock()
        .expect("active events mutex poisoned") = None;
}

fn push_event(event: FrontendEvent) {
    let ptr = *active_events()
        .lock()
        .expect("active events mutex poisoned");
    if let Some(ptr) = ptr {
        let events = ptr as *mut VecDeque<FrontendEvent>;
        unsafe { (*events).push_back(event) };
    }
}

unsafe extern "C" fn environment_callback(cmd: c_uint, _data: *mut c_void) -> bool {
    let handled = matches!(
        cmd,
        libretro::RETRO_ENVIRONMENT_GET_LOG_INTERFACE
            | libretro::RETRO_ENVIRONMENT_SET_PIXEL_FORMAT
            | libretro::RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME
    );
    push_event(FrontendEvent::EnvironmentCommand {
        command: cmd,
        handled,
    });
    handled
}

unsafe extern "C" fn video_refresh_callback(
    _data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    push_event(FrontendEvent::VideoFrame {
        width,
        height,
        pitch,
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
