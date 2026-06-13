//! Common iOS/Linux libretro core loader.

use std::{
    ffi::c_void,
    path::{Path, PathBuf},
};

use libloading::{Library, Symbol};

use crate::libretro::{sys, GameInfo, GameInfoHandle, SystemInfo};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("failed to open core library: {0}")]
    Library(#[from] libloading::Error),
    #[error("core rejected content")]
    LoadGameRejected,
    #[error("invalid game info: {0}")]
    InvalidGameInfo(#[from] std::ffi::NulError),
    #[error("core path has no file name: {0}")]
    MissingCoreName(PathBuf),
}

type RetroInit = unsafe extern "C" fn();
type RetroDeinit = unsafe extern "C" fn();
type RetroRun = unsafe extern "C" fn();
type RetroGetSystemInfo = unsafe extern "C" fn(*mut sys::retro_system_info);
type RetroLoadGame = unsafe extern "C" fn(*const sys::retro_game_info) -> bool;
type RetroUnloadGame = unsafe extern "C" fn();
type RetroSetEnvironment = unsafe extern "C" fn(sys::retro_environment_t);
type RetroSetVideoRefresh = unsafe extern "C" fn(sys::retro_video_refresh_t);
type RetroSetAudioSample = unsafe extern "C" fn(sys::retro_audio_sample_t);
type RetroSetAudioSampleBatch = unsafe extern "C" fn(sys::retro_audio_sample_batch_t);
type RetroSetInputPoll = unsafe extern "C" fn(sys::retro_input_poll_t);
type RetroSetInputState = unsafe extern "C" fn(sys::retro_input_state_t);

/// Safe owner for a dynamically loaded libretro core.
pub struct Core {
    _lib: Library,
    init: RetroInit,
    deinit: RetroDeinit,
    run: RetroRun,
    get_system_info: RetroGetSystemInfo,
    load_game: RetroLoadGame,
    unload_game: RetroUnloadGame,
    set_environment: RetroSetEnvironment,
    set_video_refresh: RetroSetVideoRefresh,
    set_audio_sample: RetroSetAudioSample,
    set_audio_sample_batch: RetroSetAudioSampleBatch,
    set_input_poll: RetroSetInputPoll,
    set_input_state: RetroSetInputState,
    loaded: bool,
}

impl Core {
    /// Load a `.so`/`.dylib` core.  This path is common to Linux and real iOS;
    /// simulator-specific branches are deliberately absent.
    pub unsafe fn open(path: impl AsRef<Path>) -> Result<Self, CoreError> {
        let lib = Library::new(path.as_ref())?;
        unsafe fn sym<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, libloading::Error> {
            let symbol: Symbol<'_, T> = lib.get(name)?;
            Ok(*symbol)
        }

        Ok(Self {
            init: sym(&lib, b"retro_init\0")?,
            deinit: sym(&lib, b"retro_deinit\0")?,
            run: sym(&lib, b"retro_run\0")?,
            get_system_info: sym(&lib, b"retro_get_system_info\0")?,
            load_game: sym(&lib, b"retro_load_game\0")?,
            unload_game: sym(&lib, b"retro_unload_game\0")?,
            set_environment: sym(&lib, b"retro_set_environment\0")?,
            set_video_refresh: sym(&lib, b"retro_set_video_refresh\0")?,
            set_audio_sample: sym(&lib, b"retro_set_audio_sample\0")?,
            set_audio_sample_batch: sym(&lib, b"retro_set_audio_sample_batch\0")?,
            set_input_poll: sym(&lib, b"retro_set_input_poll\0")?,
            set_input_state: sym(&lib, b"retro_set_input_state\0")?,
            _lib: lib,
            loaded: false,
        })
    }

    pub fn install_callbacks(&self, callbacks: CoreCallbacks) {
        unsafe {
            (self.set_environment)(callbacks.environment);
            (self.set_video_refresh)(callbacks.video_refresh);
            (self.set_audio_sample)(callbacks.audio_sample);
            (self.set_audio_sample_batch)(callbacks.audio_sample_batch);
            (self.set_input_poll)(callbacks.input_poll);
            (self.set_input_state)(callbacks.input_state);
        }
    }

    pub fn init(&self) {
        unsafe { (self.init)() }
    }

    pub fn system_info(&self) -> SystemInfo {
        let mut raw = sys::retro_system_info::default();
        unsafe {
            (self.get_system_info)(&mut raw);
            SystemInfo::from_raw(&raw)
        }
    }

    pub fn load_game(&mut self, info: GameInfo) -> Result<GameInfoHandle, CoreError> {
        let handle = GameInfoHandle::new(info)?;
        let accepted = unsafe { (self.load_game)(handle.as_ptr()) };
        if accepted {
            self.loaded = true;
            Ok(handle)
        } else {
            Err(CoreError::LoadGameRejected)
        }
    }

    pub fn run_frame(&self) {
        unsafe { (self.run)() }
    }

    pub fn open_init_with_callbacks(
        path: impl AsRef<Path>,
        callbacks: CoreCallbacks,
    ) -> Result<Self, CoreError> {
        let core_path = path.as_ref();
        let core = unsafe { Self::open(core_path)? };
        core.install_callbacks(callbacks);
        core.init();
        Ok(core)
    }

    pub fn core_display_name(path: impl AsRef<Path>) -> Result<String, CoreError> {
        path.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .map(ToOwned::to_owned)
            .ok_or_else(|| CoreError::MissingCoreName(path.as_ref().to_path_buf()))
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            if self.loaded {
                (self.unload_game)();
            }
            (self.deinit)();
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct CoreCallbacks {
    pub environment: sys::retro_environment_t,
    pub video_refresh: sys::retro_video_refresh_t,
    pub audio_sample: sys::retro_audio_sample_t,
    pub audio_sample_batch: sys::retro_audio_sample_batch_t,
    pub input_poll: sys::retro_input_poll_t,
    pub input_state: sys::retro_input_state_t,
}

pub unsafe extern "C" fn environment_default(
    _cmd: ::std::os::raw::c_uint,
    _data: *mut c_void,
) -> bool {
    false
}
