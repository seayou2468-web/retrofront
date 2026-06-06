use crate::{config::FrontendConfig, dynlib::DynamicLibrary, libretro::*};
use std::{
    ffi::{c_void, CStr, CString},
    path::{Path, PathBuf},
    ptr,
    sync::{Mutex, OnceLock},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoreMetadata {
    pub name: String,
    pub version: String,
    pub valid_extensions: String,
    pub need_fullpath: bool,
    pub supports_no_game: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameResult {
    pub width: u32,
    pub height: u32,
    pub pitch: usize,
    pub video_bytes: usize,
    pub audio_frames: usize,
}

#[derive(Debug, Default)]
struct RuntimeState {
    frame: FrameResult,
    input: [[i16; 32]; 8],
    system_dir: Option<CString>,
    save_dir: Option<CString>,
    variables: Vec<(CString, CString)>,
    supports_no_game: bool,
    variable_update: bool,
    shutdown_requested: bool,
    rotation: u32,
    messages: Vec<String>,
}

static STATE: OnceLock<Mutex<RuntimeState>> = OnceLock::new();
fn state() -> &'static Mutex<RuntimeState> {
    STATE.get_or_init(|| Mutex::new(RuntimeState::default()))
}

pub struct CoreApi {
    _library: DynamicLibrary,
    set_environment: RetroSetEnvironment,
    set_video_refresh: RetroSetVideoRefresh,
    set_audio_sample: RetroSetAudioSample,
    set_audio_sample_batch: RetroSetAudioSampleBatch,
    set_input_poll: RetroSetInputPoll,
    set_input_state: RetroSetInputState,
    init: RetroInit,
    deinit: RetroDeinit,
    api_version: RetroApiVersion,
    get_system_info: RetroGetSystemInfo,
    get_system_av_info: RetroGetSystemAvInfo,
    set_controller_port_device: RetroSetControllerPortDevice,
    reset: RetroReset,
    run: RetroRun,
    serialize_size: RetroSerializeSize,
    serialize: RetroSerialize,
    unserialize: RetroUnserialize,
    load_game: RetroLoadGame,
    _load_game_special: RetroLoadGameSpecial,
    unload_game: RetroUnloadGame,
    get_region: RetroGetRegion,
    _get_memory_data: RetroGetMemoryData,
    _get_memory_size: RetroGetMemorySize,
}

impl CoreApi {
    pub unsafe fn load(path: &Path) -> Result<Self, String> {
        let library = DynamicLibrary::open(path)?;
        Ok(Self {
            set_environment: library.symbol("retro_set_environment")?,
            set_video_refresh: library.symbol("retro_set_video_refresh")?,
            set_audio_sample: library.symbol("retro_set_audio_sample")?,
            set_audio_sample_batch: library.symbol("retro_set_audio_sample_batch")?,
            set_input_poll: library.symbol("retro_set_input_poll")?,
            set_input_state: library.symbol("retro_set_input_state")?,
            init: library.symbol("retro_init")?,
            deinit: library.symbol("retro_deinit")?,
            api_version: library.symbol("retro_api_version")?,
            get_system_info: library.symbol("retro_get_system_info")?,
            get_system_av_info: library.symbol("retro_get_system_av_info")?,
            set_controller_port_device: library.symbol("retro_set_controller_port_device")?,
            reset: library.symbol("retro_reset")?,
            run: library.symbol("retro_run")?,
            serialize_size: library.symbol("retro_serialize_size")?,
            serialize: library.symbol("retro_serialize")?,
            unserialize: library.symbol("retro_unserialize")?,
            load_game: library.symbol("retro_load_game")?,
            _load_game_special: library.symbol("retro_load_game_special")?,
            unload_game: library.symbol("retro_unload_game")?,
            get_region: library.symbol("retro_get_region")?,
            _get_memory_data: library.symbol("retro_get_memory_data")?,
            _get_memory_size: library.symbol("retro_get_memory_size")?,
            _library: library,
        })
    }
}

pub struct RetroHost {
    api: CoreApi,
    config: FrontendConfig,
    loaded: bool,
    core_path: PathBuf,
}

impl RetroHost {
    pub fn load_core(core_path: impl AsRef<Path>, config: FrontendConfig) -> Result<Self, String> {
        config.ensure_directories()?;
        let core_path = core_path.as_ref().to_path_buf();
        let api = unsafe { CoreApi::load(&core_path)? };
        {
            let mut st = state().lock().map_err(|_| "state lock poisoned")?;
            st.system_dir = Some(
                CString::new(config.paths.system_dir.to_string_lossy().as_bytes())
                    .map_err(|e| e.to_string())?,
            );
            st.save_dir = Some(
                CString::new(config.paths.save_dir.to_string_lossy().as_bytes())
                    .map_err(|e| e.to_string())?,
            );
            st.variables = config
                .core_options
                .iter()
                .map(|(k, v)| Ok((CString::new(k.as_bytes())?, CString::new(v.as_bytes())?)))
                .collect::<Result<Vec<_>, std::ffi::NulError>>()
                .map_err(|e| e.to_string())?;
        }
        unsafe {
            (api.set_environment)(environment_callback);
            (api.set_video_refresh)(video_refresh_callback);
            (api.set_audio_sample)(audio_sample_callback);
            (api.set_audio_sample_batch)(audio_sample_batch_callback);
            (api.set_input_poll)(input_poll_callback);
            (api.set_input_state)(input_state_callback);
            (api.init)();
            for port in 0..8 {
                (api.set_controller_port_device)(port, DEVICE_JOYPAD);
            }
        }
        Ok(Self {
            api,
            config,
            loaded: false,
            core_path,
        })
    }

    pub fn metadata(&self) -> CoreMetadata {
        let mut info = RetroSystemInfo::default();
        unsafe {
            (self.api.get_system_info)(&mut info);
        }
        let supports_no_game = state().lock().map(|s| s.supports_no_game).unwrap_or(false);
        CoreMetadata {
            name: cstr(info.library_name),
            version: cstr(info.library_version),
            valid_extensions: cstr(info.valid_extensions),
            need_fullpath: info.need_fullpath,
            supports_no_game,
        }
    }

    pub fn av_info(&self) -> RetroSystemAvInfo {
        let mut av = RetroSystemAvInfo::default();
        unsafe {
            (self.api.get_system_av_info)(&mut av);
        }
        av
    }

    pub fn load_game(&mut self, content_path: Option<&Path>) -> Result<(), String> {
        let path_cstring = match content_path {
            Some(p) => {
                Some(CString::new(p.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?)
            }
            None => None,
        };
        let game = RetroGameInfo {
            path: path_cstring.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            data: ptr::null(),
            size: 0,
            meta: ptr::null(),
        };
        let ok = unsafe { (self.api.load_game)(&game) };
        if ok {
            self.loaded = true;
            Ok(())
        } else {
            Err("retro_load_game returned false".into())
        }
    }

    pub fn run_frame(&mut self) -> Result<FrameResult, String> {
        if !self.loaded {
            return Err("no game is loaded".into());
        }
        {
            state().lock().map_err(|_| "state lock poisoned")?.frame = FrameResult::default();
        }
        unsafe {
            (self.api.run)();
        }
        state()
            .lock()
            .map(|s| s.frame.clone())
            .map_err(|_| "state lock poisoned".into())
    }

    pub fn reset(&self) {
        unsafe {
            (self.api.reset)();
        }
    }

    pub fn set_joypad_button(&self, port: u32, id: u32, pressed: bool) -> Result<(), String> {
        let mut st = state().lock().map_err(|_| "state lock poisoned")?;
        let Some(port_state) = st.input.get_mut(port as usize) else {
            return Err(format!("invalid input port {port}"));
        };
        let Some(slot) = port_state.get_mut(id as usize) else {
            return Err(format!("invalid joypad id {id}"));
        };
        *slot = if pressed { 1 } else { 0 };
        Ok(())
    }

    pub fn clear_input(&self) -> Result<(), String> {
        state().lock().map_err(|_| "state lock poisoned")?.input = [[0; 32]; 8];
        Ok(())
    }

    pub fn take_messages(&self) -> Vec<String> {
        state()
            .lock()
            .map(|mut s| std::mem::take(&mut s.messages))
            .unwrap_or_default()
    }

    pub fn shutdown_requested(&self) -> bool {
        state()
            .lock()
            .map(|s| s.shutdown_requested)
            .unwrap_or(false)
    }
    pub fn api_version(&self) -> u32 {
        unsafe { (self.api.api_version)() }
    }
    pub fn region(&self) -> u32 {
        unsafe { (self.api.get_region)() }
    }
    pub fn core_path(&self) -> &Path {
        &self.core_path
    }
    pub fn config(&self) -> &FrontendConfig {
        &self.config
    }

    pub fn serialize_state(&self) -> Result<Vec<u8>, String> {
        let size = unsafe { (self.api.serialize_size)() };
        let mut data = vec![0u8; size];
        if size == 0 || unsafe { (self.api.serialize)(data.as_mut_ptr().cast::<c_void>(), size) } {
            Ok(data)
        } else {
            Err("retro_serialize returned false".into())
        }
    }

    pub fn unserialize_state(&self, data: &[u8]) -> Result<(), String> {
        if unsafe { (self.api.unserialize)(data.as_ptr().cast::<c_void>(), data.len()) } {
            Ok(())
        } else {
            Err("retro_unserialize returned false".into())
        }
    }
}

impl Drop for RetroHost {
    fn drop(&mut self) {
        unsafe {
            if self.loaded {
                (self.api.unload_game)();
            }
            (self.api.deinit)();
        }
    }
}

fn cstr(pointer: *const i8) -> String {
    if pointer.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(pointer).to_string_lossy().into_owned() }
    }
}

unsafe extern "C" fn environment_callback(command: u32, data: *mut c_void) -> bool {
    let Ok(mut st) = state().lock() else {
        return false;
    };
    match command {
        ENVIRONMENT_GET_SYSTEM_DIRECTORY => write_cstr_ptr(data, st.system_dir.as_ref()),
        ENVIRONMENT_GET_SAVE_DIRECTORY => write_cstr_ptr(data, st.save_dir.as_ref()),
        ENVIRONMENT_SET_SUPPORT_NO_GAME => {
            st.supports_no_game = read_bool(data);
            true
        }
        ENVIRONMENT_GET_VARIABLE => {
            if data.is_null() {
                return false;
            }
            let variable = &mut *(data as *mut RetroVariable);
            if variable.key.is_null() {
                return false;
            }
            let key = CStr::from_ptr(variable.key).to_string_lossy();
            if let Some((_, value)) = st
                .variables
                .iter()
                .find(|(k, _)| k.to_string_lossy() == key)
            {
                variable.value = value.as_ptr();
                true
            } else {
                false
            }
        }
        ENVIRONMENT_SET_MESSAGE => {
            if data.is_null() {
                return false;
            }
            let msg = &*(data as *const RetroMessage);
            if !msg.msg.is_null() {
                st.messages
                    .push(CStr::from_ptr(msg.msg).to_string_lossy().into_owned());
            }
            true
        }
        ENVIRONMENT_SHUTDOWN => {
            st.shutdown_requested = true;
            true
        }
        ENVIRONMENT_SET_ROTATION => {
            if data.is_null() {
                false
            } else {
                st.rotation = *(data as *const u32);
                true
            }
        }
        ENVIRONMENT_GET_VARIABLE_UPDATE => {
            if data.is_null() {
                false
            } else {
                *(data as *mut bool) = st.variable_update;
                st.variable_update = false;
                true
            }
        }
        ENVIRONMENT_SET_VARIABLE => {
            if data.is_null() {
                return false;
            }
            let variable = &*(data as *const RetroVariable);
            if variable.key.is_null() || variable.value.is_null() {
                return false;
            }
            let key = CStr::from_ptr(variable.key).to_owned();
            let value = CStr::from_ptr(variable.value).to_owned();
            if let Some((_, existing)) = st
                .variables
                .iter_mut()
                .find(|(k, _)| k.as_c_str() == key.as_c_str())
            {
                *existing = value;
            } else {
                st.variables.push((key, value));
            }
            st.variable_update = true;
            true
        }
        ENVIRONMENT_SET_VARIABLES
        | ENVIRONMENT_SET_INPUT_DESCRIPTORS
        | ENVIRONMENT_SET_CONTROLLER_INFO
        | ENVIRONMENT_SET_MEMORY_MAPS
        | ENVIRONMENT_SET_HW_RENDER
        | ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE
        | ENVIRONMENT_SET_FRAME_TIME_CALLBACK
        | ENVIRONMENT_SET_AUDIO_CALLBACK
        | ENVIRONMENT_SET_SERIALIZATION_QUIRKS
        | ENVIRONMENT_SET_GEOMETRY
        | ENVIRONMENT_SET_PERFORMANCE_LEVEL
        | ENVIRONMENT_SET_KEYBOARD_CALLBACK
        | ENVIRONMENT_SET_DISK_CONTROL_INTERFACE
        | ENVIRONMENT_SET_DISK_CONTROL_EXT_INTERFACE
        | ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS
        | ENVIRONMENT_SET_CORE_OPTIONS
        | ENVIRONMENT_SET_CORE_OPTIONS_INTL
        | ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY
        | ENVIRONMENT_SET_CORE_OPTIONS_V2
        | ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL
        | ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK
        | ENVIRONMENT_SET_MESSAGE_EXT
        | ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK
        | ENVIRONMENT_SET_MINIMUM_AUDIO_LATENCY
        | ENVIRONMENT_SET_FASTFORWARDING_OVERRIDE
        | ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE => true,
        ENVIRONMENT_GET_CAN_DUPE
        | ENVIRONMENT_GET_OVERSCAN
        | ENVIRONMENT_GET_FASTFORWARDING
        | ENVIRONMENT_GET_INPUT_BITMASKS => {
            if !data.is_null() {
                *(data as *mut bool) = true;
                true
            } else {
                false
            }
        }
        ENVIRONMENT_GET_CORE_OPTIONS_VERSION
        | ENVIRONMENT_GET_MESSAGE_INTERFACE_VERSION
        | ENVIRONMENT_GET_DISK_CONTROL_INTERFACE_VERSION => {
            if !data.is_null() {
                *(data as *mut u32) = 1;
                true
            } else {
                false
            }
        }
        ENVIRONMENT_GET_INPUT_MAX_USERS => {
            if !data.is_null() {
                *(data as *mut u32) = 8;
                true
            } else {
                false
            }
        }
        ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE => {
            if !data.is_null() {
                *(data as *mut i32) = 3;
                true
            } else {
                false
            }
        }
        ENVIRONMENT_GET_TARGET_REFRESH_RATE => {
            if !data.is_null() {
                *(data as *mut f64) = 60.0;
                true
            } else {
                false
            }
        }
        ENVIRONMENT_GET_USERNAME
        | ENVIRONMENT_GET_LANGUAGE
        | ENVIRONMENT_GET_LOG_INTERFACE
        | ENVIRONMENT_GET_CURRENT_SOFTWARE_FRAMEBUFFER
        | ENVIRONMENT_GET_HW_RENDER_INTERFACE
        | ENVIRONMENT_GET_VFS_INTERFACE
        | ENVIRONMENT_GET_LED_INTERFACE
        | ENVIRONMENT_GET_MIDI_INTERFACE
        | ENVIRONMENT_GET_GAME_INFO_EXT
        | ENVIRONMENT_GET_THROTTLE_STATE
        | ENVIRONMENT_GET_SAVESTATE_CONTEXT
        | ENVIRONMENT_GET_PREFERRED_HW_RENDER
        | ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT => false,
        ENVIRONMENT_SET_PIXEL_FORMAT => true,
        _ => false,
    }
}

unsafe fn write_cstr_ptr(data: *mut c_void, value: Option<&CString>) -> bool {
    if data.is_null() {
        return false;
    }
    *(data as *mut *const i8) = value.map_or(ptr::null(), |s| s.as_ptr());
    true
}

unsafe fn read_bool(data: *mut c_void) -> bool {
    !data.is_null() && *(data as *const bool)
}

unsafe extern "C" fn video_refresh_callback(
    data: *const c_void,
    width: u32,
    height: u32,
    pitch: usize,
) {
    if let Ok(mut st) = state().lock() {
        st.frame.width = width;
        st.frame.height = height;
        st.frame.pitch = pitch;
        st.frame.video_bytes = if data.is_null() {
            0
        } else {
            pitch.saturating_mul(height as usize)
        };
    }
}
unsafe extern "C" fn audio_sample_callback(_left: i16, _right: i16) {
    if let Ok(mut st) = state().lock() {
        st.frame.audio_frames += 1;
    }
}
unsafe extern "C" fn audio_sample_batch_callback(_data: *const i16, frames: usize) -> usize {
    if let Ok(mut st) = state().lock() {
        st.frame.audio_frames += frames;
    }
    frames
}
unsafe extern "C" fn input_poll_callback() {}
unsafe extern "C" fn input_state_callback(port: u32, _device: u32, _index: u32, id: u32) -> i16 {
    state()
        .lock()
        .ok()
        .and_then(|st| {
            st.input
                .get(port as usize)
                .and_then(|p| p.get(id as usize))
                .copied()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_metadata_is_empty_for_null_cstr() {
        assert_eq!(cstr(std::ptr::null()), "");
    }
}
