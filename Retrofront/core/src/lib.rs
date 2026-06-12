//! Retrofront Rust management core.

mod assets;
mod core_info;
mod dylib;
pub mod gfx;
mod launch;
pub mod libretro;
mod menu;
mod options;
pub mod overlay;
mod playlist;
mod scanner;
mod settings;

use dylib::Library;
use gfx::{GfxBackendKind, GfxRuntime, HardwareRenderRequest, PixelFormat};
use launch::LaunchDecisionKind;
use options::CoreOptionsManager;
pub use options::{CoreOptionDefinition, CoreOptionValue};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::ffi::{CStr, CString};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub const RETRO_HW_FRAME_BUFFER_VALID: *const c_void = usize::MAX as *const c_void;

macro_rules! sym {
    ($lib:expr, $name:literal, $ty:ty) => {{
        let name =
            CStr::from_bytes_with_nul(concat!($name, "\0").as_bytes()).expect("static symbol name");
        $lib.symbol::<$ty>(name)?.get()
    }};
}

struct VfsFileHandle {
    path: CString,
    file: File,
}

struct VfsDirEntry {
    name: CString,
    is_dir: bool,
}

struct VfsDirHandle {
    entries: Vec<VfsDirEntry>,
    current: Option<usize>,
}

static PERF_EPOCH: OnceLock<Instant> = OnceLock::new();
static VFS_INTERFACE: libretro::retro_vfs_interface = libretro::retro_vfs_interface {
    get_path: Some(vfs_get_path),
    open: Some(vfs_open),
    close: Some(vfs_close),
    size: Some(vfs_size),
    tell: Some(vfs_tell),
    seek: Some(vfs_seek),
    read: Some(vfs_read),
    write: Some(vfs_write),
    flush: Some(vfs_flush),
    remove: Some(vfs_remove),
    rename: Some(vfs_rename),
    truncate: Some(vfs_truncate),
    stat: Some(vfs_stat),
    mkdir: Some(vfs_mkdir),
    opendir: Some(vfs_opendir),
    readdir: Some(vfs_readdir),
    dirent_get_name: Some(vfs_dirent_get_name),
    dirent_is_dir: Some(vfs_dirent_is_dir),
    closedir: Some(vfs_closedir),
    stat_64: Some(vfs_stat_64),
};
static PERF_INTERFACE: libretro::retro_perf_callback = libretro::retro_perf_callback {
    get_time_usec: Some(perf_get_time_usec),
    get_cpu_features: Some(perf_get_cpu_features),
    get_perf_counter: Some(perf_get_counter),
    perf_register: Some(perf_register),
    perf_start: Some(perf_start),
    perf_stop: Some(perf_stop),
    perf_log: Some(perf_log),
};
static RUMBLE_INTERFACE: libretro::retro_rumble_interface = libretro::retro_rumble_interface {
    set_rumble_state: Some(set_rumble_state),
};

type RetroSetEnvironment = unsafe extern "C" fn(libretro::retro_environment_t);
type RetroSetVideoRefresh = unsafe extern "C" fn(libretro::retro_video_refresh_t);
type RetroSetAudioSample = unsafe extern "C" fn(libretro::retro_audio_sample_t);
type RetroSetAudioSampleBatch = unsafe extern "C" fn(libretro::retro_audio_sample_batch_t);
type RetroSetInputPoll = unsafe extern "C" fn(libretro::retro_input_poll_t);
type RetroSetInputState = unsafe extern "C" fn(libretro::retro_input_state_t);
type RetroSetControllerPortDevice = unsafe extern "C" fn(c_uint, c_uint);
type RetroInit = unsafe extern "C" fn();
type RetroDeinit = unsafe extern "C" fn();
type RetroApiVersion = unsafe extern "C" fn() -> c_uint;
type RetroGetSystemInfo = unsafe extern "C" fn(*mut libretro::retro_system_info);
type RetroGetSystemAvInfo = unsafe extern "C" fn(*mut libretro::retro_system_av_info);
type RetroLoadGame = unsafe extern "C" fn(*const libretro::retro_game_info) -> bool;
type RetroUnloadGame = unsafe extern "C" fn();
type RetroRun = unsafe extern "C" fn();
type RetroReset = unsafe extern "C" fn();
type RetroSerializeSize = unsafe extern "C" fn() -> usize;
type RetroSerialize = unsafe extern "C" fn(*mut c_void, usize) -> bool;
type RetroUnserialize = unsafe extern "C" fn(*const c_void, usize) -> bool;
type RetroGetMemoryData = unsafe extern "C" fn(c_uint) -> *mut c_void;
type RetroGetMemorySize = unsafe extern "C" fn(c_uint) -> usize;

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

fn cstr_ptr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
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
    retro_set_controller_port_device: RetroSetControllerPortDevice,
    retro_init: RetroInit,
    retro_deinit: RetroDeinit,
    #[allow(dead_code)]
    retro_api_version: RetroApiVersion,
    retro_get_system_info: RetroGetSystemInfo,
    retro_get_system_av_info: RetroGetSystemAvInfo,
    retro_load_game: RetroLoadGame,
    retro_unload_game: RetroUnloadGame,
    retro_run: RetroRun,
    retro_reset: RetroReset,
    retro_serialize_size: RetroSerializeSize,
    retro_serialize: RetroSerialize,
    retro_unserialize: RetroUnserialize,
    retro_get_memory_data: RetroGetMemoryData,
    retro_get_memory_size: RetroGetMemorySize,
}

impl CoreApi {
    fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let core_path = path.as_ref();
        let lib = Library::open(core_path)
            .map_err(|e| format!("failed to load core {}: {e}", core_path.display()))?;
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
            retro_set_controller_port_device: sym!(
                lib,
                "retro_set_controller_port_device",
                RetroSetControllerPortDevice
            ),
            retro_init: sym!(lib, "retro_init", RetroInit),
            retro_deinit: sym!(lib, "retro_deinit", RetroDeinit),
            retro_api_version: sym!(lib, "retro_api_version", RetroApiVersion),
            retro_get_system_info: sym!(lib, "retro_get_system_info", RetroGetSystemInfo),
            retro_get_system_av_info: sym!(lib, "retro_get_system_av_info", RetroGetSystemAvInfo),
            retro_load_game: sym!(lib, "retro_load_game", RetroLoadGame),
            retro_unload_game: sym!(lib, "retro_unload_game", RetroUnloadGame),
            retro_run: sym!(lib, "retro_run", RetroRun),
            retro_reset: sym!(lib, "retro_reset", RetroReset),
            retro_serialize_size: sym!(lib, "retro_serialize_size", RetroSerializeSize),
            retro_serialize: sym!(lib, "retro_serialize", RetroSerialize),
            retro_unserialize: sym!(lib, "retro_unserialize", RetroUnserialize),
            retro_get_memory_data: sym!(lib, "retro_get_memory_data", RetroGetMemoryData),
            retro_get_memory_size: sym!(lib, "retro_get_memory_size", RetroGetMemorySize),
            _library: lib,
        })
    }
}

pub struct FrontendCore {
    api: Option<CoreApi>,
    system_info: Option<SystemInfo>,
    game: Option<GameInfo>,
    current_core_path: Option<PathBuf>,
    game_info_ext: Option<libretro::retro_game_info_ext>,
    game_data: Option<Vec<u8>>,
    env_strings: HashMap<String, CString>,
    events: VecDeque<FrontendEvent>,
    joypad_buttons: [i16; 16],
    pub overlay: overlay::OverlayManager,
    pub gfx: GfxRuntime,
    pub options: CoreOptionsManager,
    last_error: Option<String>,
    pub core_info: core_info::CoreInfoList,
    pub settings: settings::Settings,
    pub menu: menu::MenuEngine,
    pub scanner: scanner::Scanner,
    pub launcher: launch::LaunchManager,
}

unsafe impl Send for FrontendCore {}

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

unsafe fn vfs_file<'a>(
    stream: *mut libretro::retro_vfs_file_handle,
) -> Option<&'a mut VfsFileHandle> {
    unsafe { (stream as *mut VfsFileHandle).as_mut() }
}

unsafe fn vfs_dir<'a>(stream: *mut libretro::retro_vfs_dir_handle) -> Option<&'a mut VfsDirHandle> {
    unsafe { (stream as *mut VfsDirHandle).as_mut() }
}

unsafe extern "C" fn vfs_get_path(stream: *mut libretro::retro_vfs_file_handle) -> *const c_char {
    unsafe { vfs_file(stream) }
        .map(|handle| handle.path.as_ptr())
        .unwrap_or(ptr::null())
}

unsafe extern "C" fn vfs_open(
    path: *const c_char,
    mode: c_uint,
    _hints: c_uint,
) -> *mut libretro::retro_vfs_file_handle {
    let Some(path) = ptr_to_str(path) else {
        return ptr::null_mut();
    };
    let read = mode & libretro::RETRO_VFS_FILE_ACCESS_READ != 0;
    let write = mode & libretro::RETRO_VFS_FILE_ACCESS_WRITE != 0;
    let update_existing = mode & libretro::RETRO_VFS_FILE_ACCESS_UPDATE_EXISTING != 0;
    let mut options = OpenOptions::new();
    options.read(read || !write).write(write);
    if write && !update_existing {
        options.create(true);
        if !read {
            options.truncate(true);
        }
    }
    let Ok(file) = options.open(&path) else {
        return ptr::null_mut();
    };
    let Ok(path) = CString::new(path.replace('\0', "")) else {
        return ptr::null_mut();
    };
    Box::into_raw(Box::new(VfsFileHandle { path, file })) as *mut libretro::retro_vfs_file_handle
}

unsafe extern "C" fn vfs_close(stream: *mut libretro::retro_vfs_file_handle) -> c_int {
    if stream.is_null() {
        return -1;
    }
    let _ = unsafe { Box::from_raw(stream as *mut VfsFileHandle) };
    0
}

unsafe extern "C" fn vfs_size(stream: *mut libretro::retro_vfs_file_handle) -> i64 {
    let Some(handle) = (unsafe { vfs_file(stream) }) else {
        return -1;
    };
    handle.file.metadata().map(|m| m.len() as i64).unwrap_or(-1)
}

unsafe extern "C" fn vfs_truncate(
    stream: *mut libretro::retro_vfs_file_handle,
    length: i64,
) -> i64 {
    let Some(handle) = (unsafe { vfs_file(stream) }) else {
        return -1;
    };
    if length < 0 {
        return -1;
    }
    handle
        .file
        .set_len(length as u64)
        .map(|_| length)
        .unwrap_or(-1)
}

unsafe extern "C" fn vfs_tell(stream: *mut libretro::retro_vfs_file_handle) -> i64 {
    let Some(handle) = (unsafe { vfs_file(stream) }) else {
        return -1;
    };
    handle
        .file
        .stream_position()
        .map(|p| p as i64)
        .unwrap_or(-1)
}

unsafe extern "C" fn vfs_seek(
    stream: *mut libretro::retro_vfs_file_handle,
    offset: i64,
    seek_position: c_int,
) -> i64 {
    let Some(handle) = (unsafe { vfs_file(stream) }) else {
        return -1;
    };
    let pos = match seek_position as u32 {
        libretro::RETRO_VFS_SEEK_POSITION_START => {
            if offset < 0 {
                return -1;
            }
            SeekFrom::Start(offset as u64)
        }
        libretro::RETRO_VFS_SEEK_POSITION_CURRENT => SeekFrom::Current(offset),
        libretro::RETRO_VFS_SEEK_POSITION_END => SeekFrom::End(offset),
        _ => return -1,
    };
    handle.file.seek(pos).map(|p| p as i64).unwrap_or(-1)
}

unsafe extern "C" fn vfs_read(
    stream: *mut libretro::retro_vfs_file_handle,
    s: *mut c_void,
    len: u64,
) -> i64 {
    let Some(handle) = (unsafe { vfs_file(stream) }) else {
        return -1;
    };
    if s.is_null() {
        return -1;
    }
    let buf = unsafe { std::slice::from_raw_parts_mut(s.cast::<u8>(), len as usize) };
    handle.file.read(buf).map(|n| n as i64).unwrap_or(-1)
}

unsafe extern "C" fn vfs_write(
    stream: *mut libretro::retro_vfs_file_handle,
    s: *const c_void,
    len: u64,
) -> i64 {
    let Some(handle) = (unsafe { vfs_file(stream) }) else {
        return -1;
    };
    if s.is_null() {
        return -1;
    }
    let buf = unsafe { std::slice::from_raw_parts(s.cast::<u8>(), len as usize) };
    handle.file.write(buf).map(|n| n as i64).unwrap_or(-1)
}

unsafe extern "C" fn vfs_flush(stream: *mut libretro::retro_vfs_file_handle) -> c_int {
    let Some(handle) = (unsafe { vfs_file(stream) }) else {
        return -1;
    };
    handle.file.flush().map(|_| 0).unwrap_or(-1)
}

unsafe extern "C" fn vfs_remove(path: *const c_char) -> c_int {
    let Some(path) = ptr_to_str(path) else {
        return -1;
    };
    fs::remove_file(&path)
        .or_else(|_| fs::remove_dir(&path))
        .map(|_| 0)
        .unwrap_or(-1)
}

unsafe extern "C" fn vfs_rename(old_path: *const c_char, new_path: *const c_char) -> c_int {
    let Some(old_path) = ptr_to_str(old_path) else {
        return -1;
    };
    let Some(new_path) = ptr_to_str(new_path) else {
        return -1;
    };
    fs::rename(old_path, new_path).map(|_| 0).unwrap_or(-1)
}

unsafe extern "C" fn vfs_stat(path: *const c_char, size: *mut i32) -> c_int {
    let Some((flags, len)) = vfs_metadata(path) else {
        return -1;
    };
    if !size.is_null() {
        unsafe { *size = len.min(i32::MAX as i64) as i32 };
    }
    flags
}

unsafe extern "C" fn vfs_stat_64(path: *const c_char, size: *mut i64) -> c_int {
    let Some((flags, len)) = vfs_metadata(path) else {
        return -1;
    };
    if !size.is_null() {
        unsafe { *size = len };
    }
    flags
}

fn vfs_metadata(path: *const c_char) -> Option<(c_int, i64)> {
    let path = ptr_to_str(path)?;
    let metadata = fs::metadata(path).ok()?;
    let len = metadata.len() as i64;
    let mut flags = libretro::RETRO_VFS_STAT_IS_VALID as c_int;
    if metadata.is_dir() {
        flags |= libretro::RETRO_VFS_STAT_IS_DIRECTORY as c_int;
    }
    Some((flags, len))
}

unsafe extern "C" fn vfs_mkdir(dir: *const c_char) -> c_int {
    let Some(dir) = ptr_to_str(dir) else {
        return -1;
    };
    fs::create_dir_all(dir).map(|_| 0).unwrap_or(-1)
}

unsafe extern "C" fn vfs_opendir(
    dir: *const c_char,
    include_hidden: bool,
) -> *mut libretro::retro_vfs_dir_handle {
    let Some(dir) = ptr_to_str(dir) else {
        return ptr::null_mut();
    };
    let Ok(read_dir) = fs::read_dir(dir) else {
        return ptr::null_mut();
    };
    let mut entries = Vec::new();
    for entry in read_dir.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !include_hidden && name.starts_with('.') {
            continue;
        }
        if let Ok(name) = CString::new(name.replace('\0', "")) {
            entries.push(VfsDirEntry {
                name,
                is_dir: entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false),
            });
        }
    }
    Box::into_raw(Box::new(VfsDirHandle {
        entries,
        current: None,
    })) as *mut libretro::retro_vfs_dir_handle
}

unsafe extern "C" fn vfs_readdir(dirstream: *mut libretro::retro_vfs_dir_handle) -> bool {
    let Some(handle) = (unsafe { vfs_dir(dirstream) }) else {
        return false;
    };
    let next = handle.current.map_or(0, |current| current + 1);
    if next >= handle.entries.len() {
        return false;
    }
    handle.current = Some(next);
    true
}

unsafe extern "C" fn vfs_dirent_get_name(
    dirstream: *mut libretro::retro_vfs_dir_handle,
) -> *const c_char {
    let Some(handle) = (unsafe { vfs_dir(dirstream) }) else {
        return ptr::null();
    };
    handle
        .current
        .and_then(|current| handle.entries.get(current))
        .map(|entry| entry.name.as_ptr())
        .unwrap_or(ptr::null())
}

unsafe extern "C" fn vfs_dirent_is_dir(dirstream: *mut libretro::retro_vfs_dir_handle) -> bool {
    let Some(handle) = (unsafe { vfs_dir(dirstream) }) else {
        return false;
    };
    handle
        .current
        .and_then(|current| handle.entries.get(current))
        .is_some_and(|entry| entry.is_dir)
}

unsafe extern "C" fn vfs_closedir(dirstream: *mut libretro::retro_vfs_dir_handle) -> c_int {
    if dirstream.is_null() {
        return -1;
    }
    let _ = unsafe { Box::from_raw(dirstream as *mut VfsDirHandle) };
    0
}

unsafe extern "C" fn set_rumble_state(
    _port: c_uint,
    _effect: libretro::retro_rumble_effect,
    _strength: u16,
) -> bool {
    true
}

unsafe extern "C" fn perf_get_time_usec() -> libretro::retro_time_t {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

unsafe extern "C" fn perf_get_counter() -> libretro::retro_perf_tick_t {
    PERF_EPOCH
        .get_or_init(Instant::now)
        .elapsed()
        .as_nanos()
        .min(u64::MAX as u128) as u64
}

unsafe extern "C" fn perf_get_cpu_features() -> u64 {
    0
}

unsafe extern "C" fn perf_register(counter: *mut libretro::retro_perf_counter) {
    if let Some(counter) = unsafe { counter.as_mut() } {
        counter.registered = true;
    }
}

unsafe extern "C" fn perf_start(counter: *mut libretro::retro_perf_counter) {
    if let Some(counter) = unsafe { counter.as_mut() } {
        counter.start = unsafe { perf_get_counter() };
    }
}

unsafe extern "C" fn perf_stop(counter: *mut libretro::retro_perf_counter) {
    if let Some(counter) = unsafe { counter.as_mut() } {
        let now = unsafe { perf_get_counter() };
        counter.total = counter
            .total
            .saturating_add(now.saturating_sub(counter.start));
        counter.call_cnt = counter.call_cnt.saturating_add(1);
    }
}

unsafe extern "C" fn perf_log() {}

impl FrontendCore {
    pub fn new() -> Self {
        Self {
            api: None,
            system_info: None,
            game: None,
            current_core_path: None,
            game_info_ext: None,
            game_data: None,
            env_strings: HashMap::new(),
            events: VecDeque::new(),
            joypad_buttons: [0; 16],
            overlay: overlay::OverlayManager::new(),
            gfx: GfxRuntime::new(),
            options: CoreOptionsManager::new(),
            last_error: None,
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

    pub fn configure_from_settings(&mut self) {
        self.settings.ensure_directories();
        self.configure_overlay_from_settings();
        self.core_info
            .set_info_dir(self.settings.libretro_info_path());
        self.options.set_config_path(
            self.settings
                .path_value("core_options_path")
                .unwrap_or_else(|| self.settings.base_dir.join("retroarch-core-options.cfg")),
        );
    }

    pub fn configure_overlay_from_settings(&mut self) {
        if !self
            .settings
            .bool_value("input_overlay_enable")
            .unwrap_or(true)
        {
            self.overlay.set_enabled(false);
            return;
        }
        if let Some(path) = self.settings.string_value("input_overlay") {
            if !path.is_empty() {
                let _ = self.overlay.load(path);
            }
        }
        if let Some(opacity) = self.settings.float_value("input_overlay_opacity") {
            self.overlay.set_opacity(opacity);
        }
        if let Some(scale) = self.settings.float_value("input_overlay_scale") {
            self.overlay.set_scale_factor(scale);
        }
    }

    pub fn load_overlay(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        self.overlay.load(path)
    }

    pub fn set_overlay_touch(
        &mut self,
        slot: usize,
        x: f32,
        y: f32,
        active: bool,
    ) -> Result<(), String> {
        self.overlay.set_touch(slot, x, y, active)
    }

    pub fn clear_overlay_touches(&mut self) {
        self.overlay.clear_touches();
    }

    pub fn consume_overlay_menu_toggle(&mut self) -> bool {
        self.overlay.consume_menu_toggle()
    }

    pub fn load_core(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let core_path = path.as_ref().to_path_buf();
        self.unload_core();

        self.current_core_path = Some(core_path.clone());
        let api = CoreApi::load(&core_path)?;
        unsafe {
            (api.retro_set_environment)(Some(Self::retro_environment_callback));
            (api.retro_set_video_refresh)(Some(Self::retro_video_refresh_callback));
            (api.retro_set_audio_sample)(Some(Self::retro_audio_sample_callback));
            (api.retro_set_audio_sample_batch)(Some(Self::retro_audio_sample_batch_callback));
            (api.retro_set_input_poll)(Some(Self::retro_input_poll_callback));
            (api.retro_set_input_state)(Some(Self::retro_input_state_callback));
            (api.retro_set_controller_port_device)(0, libretro::RETRO_DEVICE_JOYPAD);
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
            library_name: cstr_ptr_to_string(sys_info.library_name),
            library_version: cstr_ptr_to_string(sys_info.library_version),
            valid_extensions: cstr_ptr_to_string(sys_info.valid_extensions)
                .split('|')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            need_fullpath: sys_info.need_fullpath,
            block_extract: sys_info.block_extract,
        });

        self.api = Some(api);
        self.last_error = None;
        Ok(())
    }

    pub fn unload_core(&mut self) {
        self.unload_game();
        if let Some(api) = self.api.take() {
            unsafe { (api.retro_deinit)() };
        }
        self.system_info = None;
        self.current_core_path = None;
        self.env_strings.clear();
        self.options.clear_definitions();
    }

    pub fn load_game(
        &mut self,
        path: impl AsRef<Path>,
        meta: Option<String>,
    ) -> Result<(), String> {
        let original_path = path.as_ref().to_path_buf();

        self.unload_game();
        self.game_data = None;

        let (retro_load_game, retro_get_system_av_info) = {
            let Some(api) = self.api.as_ref() else {
                return Err("no core loaded".to_string());
            };
            (api.retro_load_game, api.retro_get_system_av_info)
        };

        let path_buf = self.prepare_content_path_for_core(&original_path)?;
        let need_fullpath = self
            .system_info
            .as_ref()
            .map(|info| info.need_fullpath)
            .unwrap_or(true);

        if !need_fullpath {
            match fs::read(&path_buf) {
                Ok(data) => self.game_data = Some(data),
                Err(error) => {
                    self.game_data = None;
                    self.last_error = Some(format!(
                        "failed to read content into memory {}: {error}",
                        path_buf.display()
                    ));
                    return Err(self.last_error.clone().unwrap_or_default());
                }
            }
        }

        self.prepare_game_info_ext(&path_buf, meta.as_deref());
        let path_ptr = self.env_path_ptr("game_info_path", path_buf.clone());
        let meta_ptr = meta
            .as_ref()
            .map(|meta| self.env_string_ptr("game_info_meta", meta.clone()))
            .unwrap_or(ptr::null());
        let (data_ptr, data_size) = self
            .game_data
            .as_ref()
            .map(|data| (data.as_ptr().cast::<c_void>(), data.len()))
            .unwrap_or((ptr::null(), 0));

        let game_info = libretro::retro_game_info {
            path: path_ptr,
            data: data_ptr,
            size: data_size,
            meta: meta_ptr,
        };

        if unsafe { (retro_load_game)(&game_info) } {
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
            unsafe { (retro_get_system_av_info)(&mut av_info) };
            self.gfx.update_system_av_info(&av_info);
            self.load_save_ram();

            Ok(())
        } else {
            self.game_info_ext = None;
            self.game_data = None;
            let core_name = self
                .system_info
                .as_ref()
                .map(|info| info.library_name.as_str())
                .unwrap_or("core");
            Err(format!(
                "core failed to load game: {core_name} rejected {}",
                path_buf.display()
            ))
        }
    }

    fn prepare_content_path_for_core(&self, path: &Path) -> Result<PathBuf, String> {
        if !path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            return Ok(path.to_path_buf());
        }

        let Some(info) = self.system_info.as_ref() else {
            return Ok(path.to_path_buf());
        };
        if info.block_extract
            || info
                .valid_extensions
                .iter()
                .any(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            return Ok(path.to_path_buf());
        }

        self.extract_compatible_zip_member(path, &info.valid_extensions)
            .map(|extracted| extracted.unwrap_or_else(|| path.to_path_buf()))
    }

    fn extract_compatible_zip_member(
        &self,
        archive_path: &Path,
        valid_extensions: &[String],
    ) -> Result<Option<PathBuf>, String> {
        let file = File::open(archive_path).map_err(|error| {
            format!("failed to open archive {}: {error}", archive_path.display())
        })?;
        let mut archive = zip::ZipArchive::new(file).map_err(|error| {
            format!("failed to read archive {}: {error}", archive_path.display())
        })?;
        let wanted: Vec<String> = valid_extensions
            .iter()
            .map(|ext| ext.trim_start_matches('.').to_ascii_lowercase())
            .collect();
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|error| format!("failed to read archive entry {index}: {error}"))?;
            if entry.is_dir() {
                continue;
            }
            let Some(name) = Path::new(entry.name()).file_name().map(PathBuf::from) else {
                continue;
            };
            let Some(ext) = name.extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            if !wanted.iter().any(|wanted| wanted.eq_ignore_ascii_case(ext)) {
                continue;
            }
            let archive_stem = archive_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("archive");
            let out_dir = self.settings.cache_directory().join(archive_stem);
            fs::create_dir_all(&out_dir).map_err(|error| {
                format!(
                    "failed to create archive cache {}: {error}",
                    out_dir.display()
                )
            })?;
            let out_path = out_dir.join(name);
            let mut out_file = File::create(&out_path).map_err(|error| {
                format!(
                    "failed to create extracted content {}: {error}",
                    out_path.display()
                )
            })?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|error| format!("failed to extract {}: {error}", entry.name()))?;
            return Ok(Some(out_path));
        }
        Ok(None)
    }

    fn prepare_game_info_ext(&mut self, path: &Path, meta: Option<&str>) {
        let full_path = self.env_path_ptr("game_info_ext_full_path", path.to_path_buf());
        let dir = self.env_path_ptr(
            "game_info_ext_dir",
            path.parent().map(Path::to_path_buf).unwrap_or_default(),
        );
        let name = self.env_string_ptr(
            "game_info_ext_name",
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string(),
        );
        let ext = self.env_string_ptr(
            "game_info_ext_ext",
            path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default()
                .to_lowercase(),
        );
        let meta = meta
            .map(|meta| self.env_string_ptr("game_info_ext_meta", meta.to_string()))
            .unwrap_or(ptr::null());

        self.game_info_ext = Some(libretro::retro_game_info_ext {
            full_path,
            archive_path: ptr::null(),
            archive_file: ptr::null(),
            dir,
            name,
            ext,
            meta,
            data: ptr::null(),
            size: 0,
            file_in_archive: false,
            persistent_data: false,
        });
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
                if self
                    .settings
                    .get("savestate_auto_load")
                    .is_some_and(|value| value == "true")
                {
                    let _ = self.load_state(self.active_state_slot());
                }
                Ok(plan)
            }
            (LaunchDecisionKind::NeedsCoreChoice, _) => Err(plan.reason.clone()),
            (LaunchDecisionKind::NoCore, _) => Err(plan.reason.clone()),
            _ => Err("invalid launch plan".to_string()),
        }
    }

    pub fn unload_game(&mut self) {
        if self.game.is_none() {
            return;
        }
        if self
            .settings
            .get("savestate_auto_save")
            .is_some_and(|value| value == "true")
        {
            let _ = self.save_state(self.active_state_slot());
        }
        let _ = self.save_save_ram();
        if let Some(api) = self.api.as_ref() {
            unsafe { (api.retro_unload_game)() };
        }
        self.game = None;
        self.game_info_ext = None;
        self.game_data = None;
    }

    pub fn reset(&mut self) -> Result<(), String> {
        let Some(api) = self.api.as_ref() else {
            return Err("no core loaded".to_string());
        };
        if self.game.is_none() {
            return Err("no game loaded".to_string());
        }
        unsafe { (api.retro_reset)() };
        Ok(())
    }

    pub fn save_state(&mut self, slot: u32) -> Result<PathBuf, String> {
        let Some(api) = self.api.as_ref() else {
            return Err("no core loaded".to_string());
        };
        if self.game.is_none() {
            return Err("no game loaded".to_string());
        }
        let size = unsafe { (api.retro_serialize_size)() };
        if size == 0 {
            return Err("core does not support save states".to_string());
        }
        let mut data = vec![0u8; size];
        if !unsafe { (api.retro_serialize)(data.as_mut_ptr().cast(), data.len()) } {
            return Err("core failed to serialize state".to_string());
        }
        let path = self.state_path(slot)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(path)
    }

    pub fn load_state(&mut self, slot: u32) -> Result<(), String> {
        let Some(api) = self.api.as_ref() else {
            return Err("no core loaded".to_string());
        };
        if self.game.is_none() {
            return Err("no game loaded".to_string());
        }
        let path = self.state_path(slot)?;
        let data =
            fs::read(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        if unsafe { (api.retro_unserialize)(data.as_ptr().cast(), data.len()) } {
            Ok(())
        } else {
            Err("core failed to unserialize state".to_string())
        }
    }

    fn active_state_slot(&self) -> u32 {
        self.settings
            .get("state_slot")
            .and_then(|value| value.parse::<i32>().ok())
            .filter(|slot| *slot >= 0)
            .map(|slot| slot.min(999) as u32)
            .unwrap_or(0)
    }

    fn cycle_state_slot(&mut self, delta: i32) -> i32 {
        let current = self
            .settings
            .get("state_slot")
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0);
        let next = if delta > 0 {
            if current >= 999 {
                0
            } else {
                current + 1
            }
        } else if current <= 0 {
            -1
        } else {
            current - 1
        };
        self.settings.set("state_slot", &next.to_string());
        next
    }

    fn save_path(&self) -> Result<PathBuf, String> {
        let Some(game) = self.game.as_ref() else {
            return Err("no game loaded".to_string());
        };
        let stem = game
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("content");
        Ok(self
            .settings
            .savefile_directory()
            .join(format!("{stem}.srm")))
    }

    fn state_path(&self, slot: u32) -> Result<PathBuf, String> {
        let Some(game) = self.game.as_ref() else {
            return Err("no game loaded".to_string());
        };
        let stem = game
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("content");
        Ok(self
            .settings
            .savestate_directory()
            .join(format!("{stem}.state{slot}")))
    }

    fn load_save_ram(&mut self) {
        let Some(api) = self.api.as_ref() else {
            return;
        };
        let Ok(path) = self.save_path() else {
            return;
        };
        let Ok(data) = fs::read(path) else {
            return;
        };
        let size = unsafe { (api.retro_get_memory_size)(libretro::RETRO_MEMORY_SAVE_RAM) };
        let ptr = unsafe { (api.retro_get_memory_data)(libretro::RETRO_MEMORY_SAVE_RAM) };
        if size > 0 && !ptr.is_null() {
            let copy_len = size.min(data.len());
            unsafe { ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast(), copy_len) };
        }
    }

    pub fn save_save_ram(&self) -> Result<PathBuf, String> {
        let Some(api) = self.api.as_ref() else {
            return Err("no core loaded".to_string());
        };
        if self.game.is_none() {
            return Err("no game loaded".to_string());
        }
        let size = unsafe { (api.retro_get_memory_size)(libretro::RETRO_MEMORY_SAVE_RAM) };
        let ptr = unsafe { (api.retro_get_memory_data)(libretro::RETRO_MEMORY_SAVE_RAM) };
        if size == 0 || ptr.is_null() {
            return Err("core did not expose SaveRAM".to_string());
        }
        let path = self.save_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), size) };
        fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(path)
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
        if id == libretro::RETRO_DEVICE_ID_JOYPAD_MASK {
            let mut mask = 0i16;
            for button in 0..16 {
                if self.joypad_buttons[button] != 0
                    || self.overlay.joypad_button(button as u32) != 0
                {
                    mask |= 1i16 << button;
                }
            }
            mask
        } else if id < 16 {
            self.joypad_buttons[id as usize].max(self.overlay.joypad_button(id))
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

    fn env_path_ptr(&mut self, key: &str, path: PathBuf) -> *const c_char {
        let value = path.to_string_lossy().into_owned();
        self.env_string_ptr(key, value)
    }

    fn env_string_ptr(&mut self, key: &str, value: String) -> *const c_char {
        let sanitized = value.replace('\0', "");
        let entry = self
            .env_strings
            .entry(key.to_string())
            .or_insert_with(|| CString::new(sanitized.as_str()).unwrap_or_default());
        if entry.to_bytes() != sanitized.as_bytes() {
            *entry = CString::new(sanitized).unwrap_or_default();
        }
        entry.as_ptr()
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
                libretro::RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY => {
                    unsafe {
                        *(data as *mut *const c_char) =
                            core.env_path_ptr("system_directory", core.settings.system_directory())
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY => {
                    unsafe {
                        *(data as *mut *const c_char) = core
                            .env_path_ptr("savefile_directory", core.settings.savefile_directory())
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY => {
                    let path = core
                        .settings
                        .path_value("core_assets_directory")
                        .unwrap_or_else(|| core.settings.base_dir.join("downloads"));
                    unsafe {
                        *(data as *mut *const c_char) =
                            core.env_path_ptr("core_assets_directory", path)
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_LIBRETRO_PATH => {
                    let path = core.current_core_path.clone().unwrap_or_default();
                    unsafe {
                        *(data as *mut *const c_char) = core.env_path_ptr("libretro_path", path)
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME => true,
                libretro::RETRO_ENVIRONMENT_SET_CONTROLLER_INFO => true,
                libretro::RETRO_ENVIRONMENT_GET_USERNAME => {
                    unsafe {
                        *(data as *mut *const c_char) =
                            core.env_string_ptr("username", "Player".to_string())
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_LANGUAGE => {
                    unsafe {
                        *(data as *mut c_uint) = libretro::retro_language_RETRO_LANGUAGE_ENGLISH
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_HW_RENDER => {
                    let req = unsafe { &*(data as *const libretro::retro_hw_render_callback) };
                    core.gfx
                        .set_hardware_render_request(HardwareRenderRequest::from_libretro(req));
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_ROTATION => {
                    let rotation = unsafe { *(data as *const c_uint) };
                    let mut config = core.gfx.video_config();
                    config.rotation_quarters = rotation;
                    core.gfx.set_video_config(config);
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_OVERSCAN => {
                    unsafe { *(data as *mut bool) = false };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_CAN_DUPE => {
                    unsafe { *(data as *mut bool) = true };
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_MESSAGE
                | libretro::RETRO_ENVIRONMENT_SET_MESSAGE_EXT => true,
                libretro::RETRO_ENVIRONMENT_SHUTDOWN => {
                    core.last_error = Some("core requested shutdown".to_string());
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL => true,
                libretro::RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS => true,
                libretro::RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK => true,
                libretro::RETRO_ENVIRONMENT_SET_DISK_CONTROL_INTERFACE
                | libretro::RETRO_ENVIRONMENT_SET_DISK_CONTROL_EXT_INTERFACE => true,
                libretro::RETRO_ENVIRONMENT_SET_FRAME_TIME_CALLBACK
                | libretro::RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK => true,
                libretro::RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE => {
                    if !data.is_null() {
                        unsafe {
                            *(data as *mut libretro::retro_rumble_interface) = RUMBLE_INTERFACE
                        };
                    }
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_PERF_INTERFACE => {
                    if !data.is_null() {
                        unsafe { *(data as *mut libretro::retro_perf_callback) = PERF_INTERFACE };
                    }
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES => {
                    unsafe {
                        *(data as *mut u64) = (1u64 << libretro::RETRO_DEVICE_JOYPAD)
                            | (1u64 << libretro::RETRO_DEVICE_ANALOG)
                            | (1u64 << libretro::RETRO_DEVICE_POINTER)
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO => {
                    let av = unsafe { &*(data as *const libretro::retro_system_av_info) };
                    core.gfx.update_system_av_info(av);
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_SUBSYSTEM_INFO => true,
                libretro::RETRO_ENVIRONMENT_SET_MEMORY_MAPS => true,
                libretro::RETRO_ENVIRONMENT_GET_VFS_INTERFACE => {
                    if data.is_null() {
                        false
                    } else {
                        let info =
                            unsafe { &mut *(data as *mut libretro::retro_vfs_interface_info) };
                        if info.required_interface_version <= 4 {
                            info.required_interface_version = 4;
                            info.iface = (&VFS_INTERFACE as *const libretro::retro_vfs_interface)
                                as *mut libretro::retro_vfs_interface;
                            true
                        } else {
                            false
                        }
                    }
                }
                libretro::RETRO_ENVIRONMENT_SET_GEOMETRY => {
                    let geometry = unsafe { &*(data as *const libretro::retro_game_geometry) };
                    let mut config = core.gfx.video_config();
                    config.base_width = geometry.base_width;
                    config.base_height = geometry.base_height;
                    config.max_width = geometry.max_width;
                    config.max_height = geometry.max_height;
                    config.aspect_ratio = geometry.aspect_ratio;
                    core.gfx.set_video_config(config);
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS
                | libretro::RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS
                | libretro::RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT => true,
                libretro::RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE => {
                    unsafe { *(data as *mut c_uint) = 3 };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_FASTFORWARDING => {
                    unsafe { *(data as *mut bool) = false };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_TARGET_REFRESH_RATE => {
                    unsafe { *(data as *mut f32) = 0.0 };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_INPUT_BITMASKS => true,
                libretro::RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION => {
                    unsafe { *(data as *mut c_uint) = 2 };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER => {
                    unsafe {
                        *(data as *mut c_uint) =
                            libretro::retro_hw_context_type_RETRO_HW_CONTEXT_NONE
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_DISK_CONTROL_INTERFACE_VERSION => {
                    unsafe { *(data as *mut c_uint) = 1 };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_MESSAGE_INTERFACE_VERSION => {
                    unsafe { *(data as *mut c_uint) = 1 };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_INPUT_MAX_USERS => {
                    unsafe { *(data as *mut c_uint) = 1 };
                    true
                }
                libretro::RETRO_ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK
                | libretro::RETRO_ENVIRONMENT_SET_MINIMUM_AUDIO_LATENCY
                | libretro::RETRO_ENVIRONMENT_SET_FASTFORWARDING_OVERRIDE
                | libretro::RETRO_ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE
                | libretro::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK => true,
                libretro::RETRO_ENVIRONMENT_GET_GAME_INFO_EXT => {
                    if data.is_null() {
                        false
                    } else if let Some(info) = core.game_info_ext.as_ref() {
                        unsafe {
                            *(data as *mut *const libretro::retro_game_info_ext) =
                                info as *const libretro::retro_game_info_ext
                        };
                        true
                    } else {
                        false
                    }
                }
                libretro::RETRO_ENVIRONMENT_SET_VARIABLE => {
                    core.options
                        .set_definitions_v0(data as *const libretro::retro_variable);
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_THROTTLE_STATE
                | libretro::RETRO_ENVIRONMENT_GET_SAVESTATE_CONTEXT
                | libretro::RETRO_ENVIRONMENT_GET_JIT_CAPABLE
                | libretro::RETRO_ENVIRONMENT_GET_NETPLAY_CLIENT_INDEX => {
                    unsafe { *(data as *mut c_uint) = 0 };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_PLAYLIST_DIRECTORY => {
                    let path = core
                        .settings
                        .path_value("playlist_directory")
                        .unwrap_or_else(|| core.settings.base_dir.join("playlists"));
                    unsafe {
                        *(data as *mut *const c_char) =
                            core.env_path_ptr("playlist_directory", path)
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_FILE_BROWSER_START_DIRECTORY => {
                    unsafe {
                        *(data as *mut *const c_char) = core.env_path_ptr(
                            "file_browser_start_directory",
                            core.settings.content_directory(),
                        )
                    };
                    true
                }
                libretro::RETRO_ENVIRONMENT_GET_TARGET_SAMPLE_RATE => {
                    unsafe { *(data as *mut f32) = 0.0 };
                    true
                }
                libretro::RETRO_ENVIRONMENT_EXEC_MEM_ALLOC => {
                    if data.is_null() {
                        false
                    } else {
                        let request =
                            unsafe { &mut *(data as *mut libretro::retro_exec_mem_alloc) };
                        request.mode = libretro::RETRO_EXEC_MEM_MODE_UNAVAILABLE;
                        request.rx = ptr::null_mut();
                        request.rw = ptr::null_mut();
                        false
                    }
                }
                libretro::RETRO_ENVIRONMENT_EXEC_MEM_FREE => true,
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
    cached_overlay_render_descs: Vec<RfOverlayRenderDesc>,
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
pub struct RfOverlayInfo {
    pub enabled: bool,
    pub active_index: usize,
    pub overlay_count: usize,
    pub active_name: *const c_char,
}

#[repr(C)]
pub struct RfOverlayRenderDesc {
    pub image_path: *const c_char,
    pub image_index: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub alpha: f32,
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
        cached_overlay_render_descs: Vec::new(),
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
pub unsafe extern "C" fn rf_frontend_unload_core(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| core.unload_core());
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_reset(frontend: *mut RfFrontend) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    match with_active_frontend(|core| core.reset()) {
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
pub unsafe extern "C" fn rf_frontend_save_sram(frontend: *mut RfFrontend) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    match with_active_frontend(|core| core.save_save_ram()) {
        Ok(_) => {
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
pub unsafe extern "C" fn rf_frontend_save_state(frontend: *mut RfFrontend, slot: u32) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    match with_active_frontend(|core| core.save_state(slot)) {
        Ok(_) => {
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
pub unsafe extern "C" fn rf_frontend_load_state(frontend: *mut RfFrontend, slot: u32) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    match with_active_frontend(|core| core.load_state(slot)) {
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
pub unsafe extern "C" fn rf_frontend_load_overlay(
    frontend: *mut RfFrontend,
    path: *const c_char,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(path_str) = ptr_to_str(path) else {
        return false;
    };
    match with_active_frontend(|core| core.load_overlay(path_str)) {
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
pub unsafe extern "C" fn rf_frontend_set_overlay_enabled(
    _frontend: *mut RfFrontend,
    enabled: bool,
) {
    with_active_frontend(|core| core.overlay.set_enabled(enabled));
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_overlay_touch(
    frontend: *mut RfFrontend,
    slot: usize,
    x: f32,
    y: f32,
    active: bool,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    match with_active_frontend(|core| core.set_overlay_touch(slot, x, y, active)) {
        Ok(()) => true,
        Err(error) => {
            set_error(frontend, &error);
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_clear_overlay_touches(_frontend: *mut RfFrontend) {
    with_active_frontend(|core| core.clear_overlay_touches());
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_set_overlay_orientation(
    frontend: *mut RfFrontend,
    portrait: bool,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let res = with_active_frontend(|core| core.overlay.set_preferred_orientation(portrait));
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
pub unsafe extern "C" fn rf_frontend_consume_overlay_menu_toggle(
    _frontend: *mut RfFrontend,
) -> bool {
    with_active_frontend(|core| core.consume_overlay_menu_toggle())
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_overlay_info(
    frontend: *mut RfFrontend,
    out_info: *mut RfOverlayInfo,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    if out_info.is_null() {
        return false;
    }
    let (enabled, active_index, overlay_count, name) = with_active_frontend(|core| {
        (
            core.overlay.enabled(),
            core.overlay.active_index(),
            core.overlay.overlays().len(),
            core.overlay.active_name().unwrap_or("").to_string(),
        )
    });
    let active_name = cache_string(frontend, &name);
    unsafe {
        *out_info = RfOverlayInfo {
            enabled,
            active_index,
            overlay_count,
            active_name,
        };
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_overlay_render_desc_count(frontend: *mut RfFrontend) -> usize {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return 0;
    };
    frontend.cached_overlay_render_descs.clear();
    let descs = with_active_frontend(|core| core.overlay.render_descs());
    for desc in descs {
        let image_path = cache_string(frontend, &desc.image_path.to_string_lossy());
        frontend
            .cached_overlay_render_descs
            .push(RfOverlayRenderDesc {
                image_path,
                image_index: desc.image_index,
                x: desc.x,
                y: desc.y,
                w: desc.w,
                h: desc.h,
                alpha: desc.alpha,
            });
    }
    frontend.cached_overlay_render_descs.len()
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_get_overlay_render_desc(
    frontend: *mut RfFrontend,
    index: usize,
    out_desc: *mut RfOverlayRenderDesc,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_ref() }) else {
        return false;
    };
    if out_desc.is_null() {
        return false;
    }
    let Some(desc) = frontend.cached_overlay_render_descs.get(index) else {
        return false;
    };
    unsafe {
        *out_desc = RfOverlayRenderDesc {
            image_path: desc.image_path,
            image_index: desc.image_index,
            x: desc.x,
            y: desc.y,
            w: desc.w,
            h: desc.h,
            alpha: desc.alpha,
        };
    }
    true
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
        if out_len < frame.rgba.len() {
            return 0;
        }
        unsafe { ptr::copy_nonoverlapping(frame.rgba.as_ptr(), out_rgba, frame.rgba.len()) };
        frame.rgba.len()
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
            with_active_frontend(|core| {
                core.scanner.clear();
                core.scanner.scan_directory(Path::new(&dir_str), &exts);
            });
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
    with_active_frontend(|core| {
        if (100..300).contains(&action_id) {
            let index = (action_id - 100) as usize;
            let current_title = core
                .menu
                .current()
                .map(|list| list.title.clone())
                .unwrap_or_default();

            if current_title.starts_with("Choose Core for") {
                let Some(plan) = core.launcher.last_plan.clone() else {
                    core.menu
                        .push_status("Core Choice", "No pending ROM launch is available.");
                    return true;
                };
                let Some(core_info) = plan.candidates.get(index).cloned() else {
                    core.menu
                        .push_status("Core Choice", "Selected core is no longer available.");
                    return true;
                };
                core.settings
                    .set_preferred_core_for_extension(&plan.content_extension, &core_info.path);
                match core.launch_content(&plan.content_path, Some(core_info.path.clone()), None) {
                    Ok(_) => core
                        .menu
                        .push_quick_menu_with_settings(true, Some(&core.settings)),
                    Err(error) => core.menu.push_status("Launch Failed", &error),
                }
                return true;
            }

            let Some(core_info) = core.core_info.cores.get(index).cloned() else {
                return false;
            };
            match core.load_core(core_info.path.clone()) {
                Ok(_) => core
                    .menu
                    .push_status("Core Loaded", &core_info.display_name),
                Err(error) => core.menu.push_status("Core Load Failed", &error),
            }
            return true;
        }

        if (300..600).contains(&action_id) {
            let index = (action_id - 300) as usize;
            let Some(game) = core.scanner.games.get(index).cloned() else {
                return false;
            };
            if core.core_info.cores.is_empty() {
                let dir = core.settings.libretro_directory();
                core.core_info.scan_directory(&dir);
            }
            let plan = core.launcher.plan_content_launch(
                &game.path,
                &core.core_info,
                &core.settings,
                game.core_path.as_deref(),
            );
            match plan.decision {
                LaunchDecisionKind::Selected => match plan.selected_core.clone() {
                    Some(core_path) => match core.launch_content(&game.path, Some(core_path), None)
                    {
                        Ok(_) => core
                            .menu
                            .push_quick_menu_with_settings(true, Some(&core.settings)),
                        Err(error) => core.menu.push_status("Launch Failed", &error),
                    },
                    None => core
                        .menu
                        .push_status("Launch Failed", "No compatible core was selected."),
                },
                LaunchDecisionKind::NeedsCoreChoice => {
                    let candidates = plan.candidates.clone();
                    core.launcher.last_plan = Some(plan);
                    core.menu.push_core_choice(&game.label, &candidates);
                }
                LaunchDecisionKind::NoCore => {
                    core.launcher.last_plan = Some(plan.clone());
                    core.menu.push_status("No Compatible Core", &plan.reason);
                }
            }
            return true;
        }

        match action_id {
            menu::ACTION_RESUME_CONTENT => true,
            menu::ACTION_RESTART_CONTENT => {
                match core.reset() {
                    Ok(()) => core
                        .menu
                        .push_status("Restarted", "Content reset successfully."),
                    Err(error) => core.menu.push_status("Restart Failed", &error),
                }
                true
            }
            menu::ACTION_CLOSE_CONTENT | menu::ACTION_EXIT_GAME => {
                core.unload_game();
                core.menu.push_status(
                    "Game Exited",
                    "SRAM was flushed and the current game was unloaded.",
                );
                true
            }
            menu::ACTION_SAVE_STATES => {
                core.menu.push_save_state_settings(&core.settings);
                true
            }
            menu::ACTION_SAVE_STATE_SLOT_0 => {
                let slot = core.active_state_slot();
                let result = core.save_state(slot);
                match result {
                    Ok(path) => core
                        .menu
                        .push_status("State Saved", &path.to_string_lossy()),
                    Err(error) => core.menu.push_status("State Failed", &error),
                }
                true
            }
            menu::ACTION_LOAD_STATE_SLOT_0 => {
                let slot = core.active_state_slot();
                let result = core.load_state(slot);
                match result {
                    Ok(()) => core
                        .menu
                        .push_status("State Loaded", &format!("Slot {slot} was restored.")),
                    Err(error) => core.menu.push_status("Load State Failed", &error),
                }
                true
            }
            menu::ACTION_SAVE_SRAM => {
                let result = core.save_save_ram();
                match result {
                    Ok(path) => core.menu.push_status("SRAM Saved", &path.to_string_lossy()),
                    Err(error) => core.menu.push_status("SRAM Failed", &error),
                }
                true
            }
            menu::ACTION_TAKE_SCREENSHOT => {
                core.menu.push_status(
                    "Screenshot",
                    "Screenshot capture is queued from the video frame API.",
                );
                true
            }
            menu::ACTION_LOAD_CORE => {
                let cores = core.core_info.cores.clone();
                core.menu.push_core_list(&cores);
                true
            }
            menu::ACTION_LOAD_CONTENT => {
                core.menu.push_content_list(&core.scanner.games);
                true
            }
            menu::ACTION_QUICK_MENU => {
                core.menu.push_quick_menu_with_settings(
                    core.game_info().is_some(),
                    Some(&core.settings),
                );
                true
            }
            menu::ACTION_ONLINE_UPDATER => {
                core.menu.push_status(
                    "Updater",
                    "Asset and database update jobs are available through settings, not the library.",
                );
                true
            }
            menu::ACTION_SETTINGS => {
                core.menu.push_settings(&core.settings);
                true
            }
            menu::ACTION_INFORMATION | menu::ACTION_CORE_INFORMATION => {
                let system_info = core.system_info().cloned();
                let game_info = core.game_info().cloned();
                let gfx_status = core.gfx.driver_status().clone();
                core.menu
                    .push_information(system_info.as_ref(), game_info.as_ref(), &gfx_status);
                true
            }
            menu::ACTION_CONFIGURATION_FILE => {
                core.menu.push_configuration_file(&core.settings);
                true
            }
            menu::ACTION_HELP => {
                core.menu.push_help();
                true
            }
            menu::ACTION_CORE_OPTIONS | menu::ACTION_CORE_SETTINGS => {
                core.menu.push_core_settings(&core.settings);
                true
            }
            menu::ACTION_DISPLAY_SETTINGS => {
                core.menu.push_play_screen_settings(&core.settings);
                true
            }
            menu::ACTION_AUDIO_MIXER => {
                core.menu.push_audio_settings(&core.settings);
                true
            }
            menu::ACTION_CONTROLS | menu::ACTION_INPUT_MAPPING => {
                core.menu.push_input_settings(&core.settings);
                true
            }
            menu::ACTION_SHADERS => {
                core.menu.push_shader_settings(&core.settings);
                true
            }
            menu::ACTION_CHEATS => {
                core.menu.push_cheat_settings(&core.settings);
                true
            }
            menu::ACTION_OVERRIDES => {
                core.menu.push_override_settings();
                true
            }
            menu::ACTION_DISC_CONTROL => {
                core.menu.push_disc_control();
                true
            }
            menu::ACTION_REPLAY => {
                core.menu
                    .push_replay_recording_settings("Replay", &core.settings);
                true
            }
            menu::ACTION_RECORDING => {
                core.menu
                    .push_replay_recording_settings("Recording", &core.settings);
                true
            }
            menu::ACTION_STREAMING => {
                core.menu
                    .push_replay_recording_settings("Streaming", &core.settings);
                true
            }
            menu::ACTION_STATE_SLOT_DECREASE | menu::ACTION_STATE_SLOT_INCREASE => {
                let delta = if action_id == menu::ACTION_STATE_SLOT_INCREASE {
                    1
                } else {
                    -1
                };
                let slot = core.cycle_state_slot(delta);
                let label = if slot < 0 {
                    "Auto".to_string()
                } else {
                    slot.to_string()
                };
                core.menu
                    .push_status("State Slot", &format!("Active save-state slot: {label}"));
                true
            }
            menu::ACTION_UNDO_LOAD_STATE | menu::ACTION_UNDO_SAVE_STATE => {
                core.menu.push_status(
                    "Save State History",
                    "Undo entries are exposed in the quick menu and will use the save-state backup ring when implemented.",
                );
                true
            }
            menu::ACTION_ADD_TO_PLAYLIST
            | menu::ACTION_SET_CORE_ASSOCIATION
            | menu::ACTION_STATE_SLOT => {
                core.menu.push_status(
                    "Quick Menu",
                    "This RetroArch quick-menu entry is modeled and ready for a platform callback.",
                );
                true
            }
            menu::ACTION_SETTINGS_DRIVERS => {
                core.menu.push_driver_settings(&core.settings);
                true
            }
            menu::ACTION_SETTINGS_VIDEO => {
                core.menu.push_video_settings(&core.settings);
                true
            }
            menu::ACTION_SETTINGS_AUDIO => {
                core.menu.push_audio_settings(&core.settings);
                true
            }
            menu::ACTION_SETTINGS_INPUT => {
                core.menu.push_input_settings(&core.settings);
                true
            }
            menu::ACTION_SETTINGS_USER_INTERFACE => {
                core.menu.push_skin_settings(&core.settings);
                true
            }
            menu::ACTION_SKIN_SETTINGS => {
                let current = core
                    .settings
                    .get("menu_driver")
                    .map_or("oneui", String::as_str);
                let next = menu::MenuDriver::next_ident(current);
                core.settings.set("menu_driver", next);
                core.configure_from_settings();
                core.menu.push_skin_settings(&core.settings);
                true
            }
            value if value == menu::ACTION_SKIN_SETTINGS + 5 => {
                let current = core
                    .settings
                    .get("menu_theme")
                    .map_or("dark", String::as_str);
                let next = match current {
                    "dark" => "light",
                    "light" => "auto",
                    "auto" => "high_contrast",
                    _ => "dark",
                };
                core.settings.set("menu_theme", next);
                core.menu.push_skin_settings(&core.settings);
                true
            }
            270..=274 => {
                let driver = core
                    .settings
                    .get("menu_driver")
                    .map_or("oneui", String::as_str);
                let (key, values): (&str, &[&str]) = match (driver, action_id) {
                    ("ozone", 270) => ("ozone_show_sidebar", &["true", "false"]),
                    ("ozone", 271) => (
                        "ozone_header_style",
                        &["icon_separator", "icon", "separator", "none"],
                    ),
                    ("ozone", 272) => ("ozone_padding_factor", &["0.75", "1.0", "1.25", "1.5"]),
                    ("ozone", 273) => ("ozone_font_scale", &["0.85", "1.0", "1.15", "1.30"]),
                    ("ozone", 274) => (
                        "ozone_thumbnail_scale_factor",
                        &["0.75", "1.0", "1.25", "1.5"],
                    ),
                    ("materialui", 270) => ("materialui_icons_enable", &["true", "false"]),
                    ("materialui", 271) => ("materialui_switch_icons", &["true", "false"]),
                    ("materialui", 272) => ("materialui_show_nav_bar", &["true", "false"]),
                    ("materialui", 273) => ("materialui_auto_rotate_nav_bar", &["true", "false"]),
                    ("materialui", 274) => (
                        "materialui_dual_thumbnail_list_view_enable",
                        &["true", "false"],
                    ),
                    ("rgui", 270) => (
                        "rgui_menu_theme_preset",
                        &["default", "classic", "blue", "gruvbox", "solarized"],
                    ),
                    ("rgui", 271) => ("rgui_aspect_ratio", &["auto", "4:3", "16:9", "16:10"]),
                    ("rgui", 272) => ("rgui_inline_thumbnails", &["false", "true"]),
                    ("rgui", 273) => ("rgui_extended_ascii", &["true", "false"]),
                    ("rgui", 274) => ("rgui_full_width_layout", &["true", "false"]),
                    ("xmb", 270) => (
                        "xmb_theme",
                        &["monochrome", "flatui", "systematic", "automatic"],
                    ),
                    ("xmb", 271) => ("xmb_show_horizontal_list", &["true", "false"]),
                    ("xmb", 272) => ("xmb_shadows_enable", &["true", "false"]),
                    ("xmb", 273) => ("xmb_alpha_factor", &["35", "50", "75", "100"]),
                    ("xmb", 274) => ("xmb_layout", &["auto", "desktop", "handheld", "console"]),
                    (_, 270) => (
                        "menu_layout_density",
                        &["compact", "standard", "comfortable"],
                    ),
                    (_, 271) => ("menu_card_style", &["modern", "flat", "outlined"]),
                    (_, 272) => (
                        "quick_menu_style",
                        &["oneui_fullscreen", "bottom_sheet", "compact"],
                    ),
                    _ => ("", &[]),
                };
                if !key.is_empty() && !values.is_empty() {
                    let current = core.settings.get(key).map_or(values[0], String::as_str);
                    let index = values
                        .iter()
                        .position(|value| *value == current)
                        .unwrap_or(0);
                    core.settings.set(key, values[(index + 1) % values.len()]);
                    core.menu.push_skin_settings(&core.settings);
                    true
                } else {
                    false
                }
            }
            menu::ACTION_SETTINGS_DIRECTORIES => {
                core.menu.push_directory_settings(&core.settings);
                true
            }
            menu::ACTION_SETTINGS_SAVING => {
                core.menu.push_save_state_settings(&core.settings);
                true
            }
            menu::ACTION_SETTINGS_LATENCY => {
                core.menu.push_placeholder_settings("Latency");
                true
            }
            menu::ACTION_SETTINGS_FRAME_THROTTLE => {
                core.menu.push_placeholder_settings("Frame Throttle");
                true
            }
            menu::ACTION_SETTINGS_PLAYLISTS => {
                core.menu.push_library_settings(&core.settings);
                true
            }
            menu::ACTION_SETTINGS_PLAY_SCREEN => {
                core.menu.push_play_screen_settings(&core.settings);
                true
            }
            menu::ACTION_SETTINGS_LIBRARY => {
                core.menu.push_library_settings(&core.settings);
                true
            }
            menu::ACTION_SETTINGS_CORE => {
                core.menu.push_core_settings(&core.settings);
                true
            }
            _ => false,
        }
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
        core.settings.set(
            "libretro_directory",
            &base_dir.join("Cores").to_string_lossy(),
        );
        core.settings.set(
            "libretro_info_path",
            &base_dir.join("info").to_string_lossy(),
        );
        core.settings.set(
            "core_options_path",
            &base_dir
                .join("retroarch-core-options.cfg")
                .to_string_lossy(),
        );
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

fn cache_string(frontend: &mut RfFrontend, value: &str) -> *const c_char {
    let c_value = CString::new(value).unwrap_or_default();
    let ptr = c_value.as_ptr();
    frontend.cached_strings.push(c_value);
    ptr
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
    fn environment_dispatch_handles_experimental_vfs_command() {
        let mut info = libretro::retro_vfs_interface_info {
            required_interface_version: 4,
            iface: ptr::null_mut(),
        };

        let ok = unsafe {
            FrontendCore::retro_environment_callback(
                libretro::RETRO_ENVIRONMENT_GET_VFS_INTERFACE,
                (&mut info as *mut libretro::retro_vfs_interface_info).cast(),
            )
        };

        assert!(ok);
        assert_eq!(info.required_interface_version, 4);
        assert!(!info.iface.is_null());
    }

    #[test]
    fn vfs_callbacks_roundtrip_file_data() {
        let dir = std::env::temp_dir().join(format!(
            "retrofront-vfs-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("save.bin");
        let c_path = CString::new(path.to_string_lossy().as_bytes()).unwrap();

        let stream = unsafe {
            vfs_open(
                c_path.as_ptr(),
                libretro::RETRO_VFS_FILE_ACCESS_READ_WRITE,
                libretro::RETRO_VFS_FILE_ACCESS_HINT_NONE,
            )
        };
        assert!(!stream.is_null());

        let payload = b"retrofront";
        assert_eq!(
            unsafe { vfs_write(stream, payload.as_ptr().cast(), payload.len() as u64) },
            payload.len() as i64
        );
        assert_eq!(
            unsafe { vfs_seek(stream, 0, libretro::RETRO_VFS_SEEK_POSITION_START as c_int) },
            0
        );
        let mut readback = vec![0_u8; payload.len()];
        assert_eq!(
            unsafe { vfs_read(stream, readback.as_mut_ptr().cast(), readback.len() as u64) },
            payload.len() as i64
        );
        assert_eq!(readback, payload);
        assert_eq!(unsafe { vfs_close(stream) }, 0);
        let _ = fs::remove_dir_all(dir);
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
#[repr(C)]
pub struct RfAssetInstallReport {
    files_written: usize,
    directories_created: usize,
}

#[no_mangle]
pub unsafe extern "C" fn rf_frontend_install_assets_zip(
    frontend: *mut RfFrontend,
    zip_path: *const c_char,
    destination_dir: *const c_char,
    out_report: *mut RfAssetInstallReport,
) -> bool {
    let Some(frontend) = (unsafe { frontend.as_mut() }) else {
        return false;
    };
    let Some(zip_path) = ptr_to_str(zip_path) else {
        return false;
    };
    let Some(destination_dir) = ptr_to_str(destination_dir) else {
        return false;
    };
    match assets::install_assets_zip(Path::new(&zip_path), Path::new(&destination_dir)) {
        Ok(report) => {
            if let Some(out) = unsafe { out_report.as_mut() } {
                out.files_written = report.files_written;
                out.directories_created = report.directories_created;
            }
            true
        }
        Err(err) => {
            set_error(frontend, &err);
            false
        }
    }
}
