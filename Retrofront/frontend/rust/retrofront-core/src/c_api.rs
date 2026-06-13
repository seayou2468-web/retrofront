//! Thin C ABI consumed by `Retrofront/frontend/menu` and platform shells.

use std::{
    ffi::{c_char, c_void, CStr},
    path::{Path, PathBuf},
    ptr, slice,
    sync::OnceLock,
};

use parking_lot::Mutex;

use crate::{
    core::{environment_default, Core, CoreCallbacks},
    input::{InputEvent, InputSource, MenuAction},
    libretro::{GameInfo, GameInfoHandle},
    menu::MenuEntry,
    renderer::{PixelFormat, VideoFrame},
    settings::SettingValue,
    RetrofrontRuntime,
};

static RUNTIME: OnceLock<RetrofrontRuntime> = OnceLock::new();
static CORE_SESSION: Mutex<Option<CoreSession>> = Mutex::new(None);
static PIXEL_FORMAT: Mutex<PixelFormat> = Mutex::new(PixelFormat::Xrgb8888);

struct CoreSession {
    core: Core,
    _game: Option<GameInfoHandle>,
}

// The C ABI owns the core session behind a global mutex and all libretro calls
// are serialized through that mutex. GameInfoHandle contains raw pointers into
// its owned buffers, so Rust cannot derive Send even though it is never moved
// without the mutex guarding it.
unsafe impl Send for CoreSession {}

fn runtime() -> Option<&'static RetrofrontRuntime> {
    RUNTIME.get()
}

/// Initialize all Rust-owned menu dependencies.
#[no_mangle]
pub extern "C" fn retrofront_runtime_init(data_dir: *const c_char) -> bool {
    if data_dir.is_null() {
        return false;
    }
    let data_dir = unsafe { CStr::from_ptr(data_dir) }
        .to_string_lossy()
        .into_owned();
    let runtime = RetrofrontRuntime::new(PathBuf::from(data_dir));
    if runtime.filesystem.ensure_layout().is_err() || runtime.settings.load().is_err() {
        return false;
    }
    RUNTIME.set(runtime).is_ok()
}

#[no_mangle]
pub extern "C" fn retrofront_menu_api_version() -> u32 {
    2
}

#[no_mangle]
pub extern "C" fn retrofront_menu_set_title(title: *const c_char) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    let Some(title) = cstr(title) else {
        return false;
    };
    let entries = runtime.menu.read().current_entries().to_vec();
    runtime.menu.write().set_root(title, entries);
    true
}

#[no_mangle]
pub extern "C" fn retrofront_menu_clear_entries() -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    let title = runtime.menu.read().title().to_owned();
    runtime.menu.write().set_root(title, Vec::new());
    true
}

#[no_mangle]
pub extern "C" fn retrofront_menu_append_entry(label: *const c_char, path: *const c_char) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    let Some(label) = cstr(label) else {
        return false;
    };
    let path = cstr(path).unwrap_or_default();
    let mut menu = runtime.menu.write();
    let title = menu.title().to_owned();
    let mut entries = menu.current_entries().to_vec();
    entries.push(MenuEntry {
        label,
        path,
        ..Default::default()
    });
    menu.set_root(title, entries);
    true
}

#[no_mangle]
pub extern "C" fn retrofront_menu_entry_count() -> usize {
    runtime()
        .map(|r| r.menu.read().current_entries().len())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn retrofront_menu_selected_index() -> usize {
    runtime()
        .map(|r| r.menu.read().current_selection())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn retrofront_menu_draw() -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    let menu = runtime.menu.read().clone();
    runtime
        .renderer
        .write()
        .draw_menu(&menu, &mut runtime.shaders.write());
    true
}

#[no_mangle]
pub extern "C" fn retrofront_input_bind_key(key: u32, action: u32) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    let Some(action) = action_from_u32(action) else {
        return false;
    };
    runtime.input.write().bind(InputSource::Key(key), action);
    true
}

#[no_mangle]
pub extern "C" fn retrofront_input_push_key(key: u32, pressed: bool) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    runtime.input.write().push_event(InputEvent {
        source: InputSource::Key(key),
        pressed,
    });
    true
}

#[no_mangle]
pub extern "C" fn retrofront_input_push_gamepad_button(port: u8, id: u16, pressed: bool) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    runtime.input.write().push_event(InputEvent {
        source: InputSource::GamepadButton { port, id },
        pressed,
    });
    true
}

#[no_mangle]
pub extern "C" fn retrofront_input_set_analog(
    port: u8,
    device: u32,
    index: u32,
    value: i16,
) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    runtime.input.write().set_analog(port, device, index, value);
    true
}

#[no_mangle]
pub extern "C" fn retrofront_menu_pump_input() -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    while let Some(action) = runtime.input.write().next_action() {
        runtime.menu.write().action(action);
    }
    true
}

#[no_mangle]
pub extern "C" fn retrofront_renderer_resize(width: u32, height: u32) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    runtime.renderer.write().resize(width, height);
    true
}

#[no_mangle]
pub extern "C" fn retrofront_shader_set_preset(path: *const c_char) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    let Some(path) = cstr(path) else {
        return false;
    };
    runtime.shaders.write().set_preset(path).is_ok()
}

#[no_mangle]
pub extern "C" fn retrofront_resources_unpack(zip_path: *const c_char) -> usize {
    let Some(runtime) = runtime() else {
        return 0;
    };
    let Some(zip_path) = cstr(zip_path) else {
        return 0;
    };
    runtime
        .filesystem
        .unpack_resources_zip(zip_path)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn retrofront_import_rom(path: *const c_char, playlist: *const c_char) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    let Some(path) = cstr(path) else {
        return false;
    };
    let playlist = cstr(playlist).unwrap_or_else(|| "Imported".to_owned());
    let imported = runtime
        .filesystem
        .copy_into_imports(&path)
        .unwrap_or_else(|_| PathBuf::from(path));
    runtime
        .playlists
        .import_rom_entry(&playlist, imported, None, None)
        .is_ok()
}

#[no_mangle]
pub extern "C" fn retrofront_settings_set_string(key: *const c_char, value: *const c_char) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    let Some(key) = cstr(key) else {
        return false;
    };
    let Some(value) = cstr(value) else {
        return false;
    };
    runtime.settings.set(key, SettingValue::String(value));
    runtime.settings.save().is_ok()
}

#[no_mangle]
pub extern "C" fn retrofront_core_open(core_path: *const c_char) -> bool {
    let Some(core_path) = cstr(core_path) else {
        return false;
    };
    let callbacks = CoreCallbacks {
        environment: Some(retrofront_environment),
        video_refresh: Some(retrofront_video_refresh),
        audio_sample: Some(retrofront_audio_sample),
        audio_sample_batch: Some(retrofront_audio_sample_batch),
        input_poll: Some(retrofront_input_poll),
        input_state: Some(retrofront_input_state),
    };
    match Core::open_init_with_callbacks(Path::new(&core_path), callbacks) {
        Ok(core) => {
            *CORE_SESSION.lock() = Some(CoreSession { core, _game: None });
            true
        }
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn retrofront_core_load_game(game_path: *const c_char) -> bool {
    let Some(game_path) = cstr(game_path) else {
        return false;
    };
    let mut guard = CORE_SESSION.lock();
    let Some(session) = guard.as_mut() else {
        return false;
    };
    match session.core.load_game(GameInfo {
        path: Some(game_path.into()),
        data: None,
        meta: None,
    }) {
        Ok(handle) => {
            session._game = Some(handle);
            true
        }
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn retrofront_core_run_frame() -> bool {
    let guard = CORE_SESSION.lock();
    let Some(session) = guard.as_ref() else {
        return false;
    };
    session.core.run_frame();
    true
}

#[no_mangle]
pub extern "C" fn retrofront_tick() {
    if let Some(runtime) = runtime() {
        runtime.tick();
    }
}

#[no_mangle]
pub extern "C" fn retrofront_runtime_shutdown() {
    *CORE_SESSION.lock() = None;
}

unsafe extern "C" fn retrofront_environment(
    cmd: ::std::os::raw::c_uint,
    data: *mut c_void,
) -> bool {
    if cmd == 10 && !data.is_null() {
        let format = *(data as *const u32);
        *PIXEL_FORMAT.lock() = match format {
            1 => PixelFormat::Xrgb8888,
            2 => PixelFormat::Rgb565,
            other => PixelFormat::Unknown(other),
        };
        return true;
    }
    environment_default(cmd, data)
}

unsafe extern "C" fn retrofront_video_refresh(
    data: *const c_void,
    width: u32,
    height: u32,
    pitch: usize,
) {
    let Some(runtime) = runtime() else {
        return;
    };
    if data.is_null() || width == 0 || height == 0 || pitch == 0 {
        return;
    }
    let byte_len = pitch.saturating_mul(height as usize);
    let bytes = slice::from_raw_parts(data.cast::<u8>(), byte_len).to_vec();
    runtime.renderer.write().submit_libretro_frame(VideoFrame {
        width,
        height,
        pitch,
        format: *PIXEL_FORMAT.lock(),
        bytes,
    });
}

unsafe extern "C" fn retrofront_audio_sample(_left: i16, _right: i16) {}
unsafe extern "C" fn retrofront_audio_sample_batch(_data: *const i16, frames: usize) -> usize {
    frames
}
unsafe extern "C" fn retrofront_input_poll() {}
unsafe extern "C" fn retrofront_input_state(port: u32, device: u32, index: u32, id: u32) -> i16 {
    let Some(runtime) = runtime() else {
        return 0;
    };
    if device == 1 {
        runtime
            .input
            .read()
            .libretro_button_state(port as u8, id as u16)
    } else {
        runtime
            .input
            .read()
            .libretro_analog_state(port as u8, device, index)
    }
}

fn action_from_u32(action: u32) -> Option<MenuAction> {
    Some(match action {
        0 => MenuAction::Up,
        1 => MenuAction::Down,
        2 => MenuAction::Left,
        3 => MenuAction::Right,
        4 => MenuAction::Ok,
        5 => MenuAction::Cancel,
        6 => MenuAction::Start,
        7 => MenuAction::Select,
        8 => MenuAction::Info,
        9 => MenuAction::Scan,
        _ => return None,
    })
}

fn cstr(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

#[allow(dead_code)]
fn copy_cstr(src: &str, dst: *mut c_char, dst_len: usize) -> bool {
    if dst.is_null() || dst_len == 0 {
        return false;
    }
    let bytes = src.as_bytes();
    let count = bytes.len().min(dst_len - 1);
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), dst.cast::<u8>(), count);
        *dst.add(count) = 0;
    }
    true
}
