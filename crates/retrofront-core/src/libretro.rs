use std::ffi::{c_char, c_uint, c_void};

pub type RetroEnvironment = unsafe extern "C" fn(c_uint, *mut c_void) -> bool;
pub type RetroVideoRefresh = unsafe extern "C" fn(*const c_void, u32, u32, usize);
pub type RetroAudioSample = unsafe extern "C" fn(i16, i16);
pub type RetroAudioSampleBatch = unsafe extern "C" fn(*const i16, usize) -> usize;
pub type RetroInputPoll = unsafe extern "C" fn();
pub type RetroInputState = unsafe extern "C" fn(u32, u32, u32, u32) -> i16;

pub const ENVIRONMENT_SET_PIXEL_FORMAT: c_uint = 10;
pub const ENVIRONMENT_GET_SYSTEM_DIRECTORY: c_uint = 9;
pub const ENVIRONMENT_GET_SAVE_DIRECTORY: c_uint = 31;
pub const ENVIRONMENT_SET_SUPPORT_NO_GAME: c_uint = 18;
pub const ENVIRONMENT_GET_VARIABLE: c_uint = 15;
pub const ENVIRONMENT_SET_VARIABLES: c_uint = 16;
pub const ENVIRONMENT_GET_LOG_INTERFACE: c_uint = 27;
pub const ENVIRONMENT_SET_CONTROLLER_INFO: c_uint = 35;
pub const ENVIRONMENT_SET_MEMORY_MAPS: c_uint = 36;
pub const ENVIRONMENT_GET_USERNAME: c_uint = 38;
pub const ENVIRONMENT_GET_LANGUAGE: c_uint = 39;
pub const ENVIRONMENT_SET_INPUT_DESCRIPTORS: c_uint = 11;
pub const ENVIRONMENT_SET_HW_RENDER: c_uint = 14;
pub const ENVIRONMENT_GET_CAN_DUPE: c_uint = 3;
pub const ENVIRONMENT_SET_FRAME_TIME_CALLBACK: c_uint = 21;
pub const ENVIRONMENT_SET_AUDIO_CALLBACK: c_uint = 22;
pub const ENVIRONMENT_SET_SERIALIZATION_QUIRKS: c_uint = 44;
pub const ENVIRONMENT_SET_GEOMETRY: c_uint = 37;

pub const DEVICE_JOYPAD: u32 = 1;
pub const DEVICE_ANALOG: u32 = 5;
pub const REGION_NTSC: u32 = 0;
pub const REGION_PAL: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RetroGameGeometry {
    pub base_width: u32,
    pub base_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub aspect_ratio: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RetroSystemTiming {
    pub fps: f64,
    pub sample_rate: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RetroSystemAvInfo {
    pub geometry: RetroGameGeometry,
    pub timing: RetroSystemTiming,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetroSystemInfo {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

impl Default for RetroSystemInfo {
    fn default() -> Self {
        Self {
            library_name: std::ptr::null(),
            library_version: std::ptr::null(),
            valid_extensions: std::ptr::null(),
            need_fullpath: false,
            block_extract: false,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetroGameInfo {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub meta: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetroVariable {
    pub key: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetroLogCallback {
    pub log: Option<unsafe extern "C" fn(u32, *const c_char, ...)>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetroInputDescriptor {
    pub port: u32,
    pub device: u32,
    pub index: u32,
    pub id: u32,
    pub description: *const c_char,
}

pub type RetroSetEnvironment = unsafe extern "C" fn(RetroEnvironment);
pub type RetroSetVideoRefresh = unsafe extern "C" fn(RetroVideoRefresh);
pub type RetroSetAudioSample = unsafe extern "C" fn(RetroAudioSample);
pub type RetroSetAudioSampleBatch = unsafe extern "C" fn(RetroAudioSampleBatch);
pub type RetroSetInputPoll = unsafe extern "C" fn(RetroInputPoll);
pub type RetroSetInputState = unsafe extern "C" fn(RetroInputState);
pub type RetroInit = unsafe extern "C" fn();
pub type RetroDeinit = unsafe extern "C" fn();
pub type RetroApiVersion = unsafe extern "C" fn() -> u32;
pub type RetroGetSystemInfo = unsafe extern "C" fn(*mut RetroSystemInfo);
pub type RetroGetSystemAvInfo = unsafe extern "C" fn(*mut RetroSystemAvInfo);
pub type RetroSetControllerPortDevice = unsafe extern "C" fn(u32, u32);
pub type RetroReset = unsafe extern "C" fn();
pub type RetroRun = unsafe extern "C" fn();
pub type RetroSerializeSize = unsafe extern "C" fn() -> usize;
pub type RetroSerialize = unsafe extern "C" fn(*mut c_void, usize) -> bool;
pub type RetroUnserialize = unsafe extern "C" fn(*const c_void, usize) -> bool;
pub type RetroLoadGame = unsafe extern "C" fn(*const RetroGameInfo) -> bool;
pub type RetroLoadGameSpecial = unsafe extern "C" fn(u32, *const RetroGameInfo, usize) -> bool;
pub type RetroUnloadGame = unsafe extern "C" fn();
pub type RetroGetRegion = unsafe extern "C" fn() -> u32;
pub type RetroGetMemoryData = unsafe extern "C" fn(u32) -> *mut c_void;
pub type RetroGetMemorySize = unsafe extern "C" fn(u32) -> usize;

pub const DEVICE_ID_JOYPAD_B: u32 = 0;
pub const DEVICE_ID_JOYPAD_Y: u32 = 1;
pub const DEVICE_ID_JOYPAD_SELECT: u32 = 2;
pub const DEVICE_ID_JOYPAD_START: u32 = 3;
pub const DEVICE_ID_JOYPAD_UP: u32 = 4;
pub const DEVICE_ID_JOYPAD_DOWN: u32 = 5;
pub const DEVICE_ID_JOYPAD_LEFT: u32 = 6;
pub const DEVICE_ID_JOYPAD_RIGHT: u32 = 7;
pub const DEVICE_ID_JOYPAD_A: u32 = 8;
pub const DEVICE_ID_JOYPAD_X: u32 = 9;
pub const DEVICE_ID_JOYPAD_L: u32 = 10;
pub const DEVICE_ID_JOYPAD_R: u32 = 11;
pub const DEVICE_ID_JOYPAD_L2: u32 = 12;
pub const DEVICE_ID_JOYPAD_R2: u32 = 13;
pub const DEVICE_ID_JOYPAD_L3: u32 = 14;
pub const DEVICE_ID_JOYPAD_R3: u32 = 15;

pub const ENVIRONMENT_GET_OVERSCAN: c_uint = 2;
pub const ENVIRONMENT_SET_MESSAGE: c_uint = 6;
pub const ENVIRONMENT_SHUTDOWN: c_uint = 7;
pub const ENVIRONMENT_SET_PERFORMANCE_LEVEL: c_uint = 8;
pub const ENVIRONMENT_SET_KEYBOARD_CALLBACK: c_uint = 12;
pub const ENVIRONMENT_SET_DISK_CONTROL_INTERFACE: c_uint = 13;
pub const ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE: c_uint = 43;
pub const ENVIRONMENT_GET_VARIABLE_UPDATE: c_uint = 17;
pub const ENVIRONMENT_SET_ROTATION: c_uint = 1;
pub const ENVIRONMENT_GET_CURRENT_SOFTWARE_FRAMEBUFFER: c_uint = 40;
pub const ENVIRONMENT_GET_HW_RENDER_INTERFACE: c_uint = 41;
pub const ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS: c_uint = 42;
pub const ENVIRONMENT_GET_VFS_INTERFACE: c_uint = 45;
pub const ENVIRONMENT_GET_LED_INTERFACE: c_uint = 46;
pub const ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE: c_uint = 47;
pub const ENVIRONMENT_GET_MIDI_INTERFACE: c_uint = 48;
pub const ENVIRONMENT_GET_FASTFORWARDING: c_uint = 49;
pub const ENVIRONMENT_GET_TARGET_REFRESH_RATE: c_uint = 50;
pub const ENVIRONMENT_GET_INPUT_BITMASKS: c_uint = 51;
pub const ENVIRONMENT_GET_CORE_OPTIONS_VERSION: c_uint = 52;
pub const ENVIRONMENT_SET_CORE_OPTIONS: c_uint = 53;
pub const ENVIRONMENT_SET_CORE_OPTIONS_INTL: c_uint = 54;
pub const ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY: c_uint = 55;
pub const ENVIRONMENT_GET_PREFERRED_HW_RENDER: c_uint = 56;
pub const ENVIRONMENT_GET_DISK_CONTROL_INTERFACE_VERSION: c_uint = 57;
pub const ENVIRONMENT_SET_DISK_CONTROL_EXT_INTERFACE: c_uint = 58;
pub const ENVIRONMENT_GET_MESSAGE_INTERFACE_VERSION: c_uint = 59;
pub const ENVIRONMENT_SET_MESSAGE_EXT: c_uint = 60;
pub const ENVIRONMENT_GET_INPUT_MAX_USERS: c_uint = 61;
pub const ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK: c_uint = 62;
pub const ENVIRONMENT_SET_MINIMUM_AUDIO_LATENCY: c_uint = 63;
pub const ENVIRONMENT_SET_FASTFORWARDING_OVERRIDE: c_uint = 64;
pub const ENVIRONMENT_SET_CONTENT_INFO_OVERRIDE: c_uint = 65;
pub const ENVIRONMENT_GET_GAME_INFO_EXT: c_uint = 66;
pub const ENVIRONMENT_SET_CORE_OPTIONS_V2: c_uint = 67;
pub const ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL: c_uint = 68;
pub const ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK: c_uint = 69;
pub const ENVIRONMENT_SET_VARIABLE: c_uint = 70;
pub const ENVIRONMENT_GET_THROTTLE_STATE: c_uint = 71;
pub const ENVIRONMENT_GET_SAVESTATE_CONTEXT: c_uint = 72;
pub const ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT: c_uint = 73;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetroMessage {
    pub msg: *const c_char,
    pub frames: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetroKeyboardCallback {
    pub callback: Option<unsafe extern "C" fn(bool, u32, u32, u16)>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetroControllerDescription {
    pub desc: *const c_char,
    pub id: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RetroControllerInfo {
    pub types: *const RetroControllerDescription,
    pub num_types: u32,
}
