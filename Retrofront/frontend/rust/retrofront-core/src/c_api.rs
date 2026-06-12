//! Thin C ABI consumed by `Retrofront/frontend/menu`.

use std::{
    ffi::{c_char, CStr},
    path::PathBuf,
    ptr,
    sync::OnceLock,
};

use crate::{
    input::{InputEvent, InputSource, MenuAction},
    menu::MenuEntry,
    RetrofrontRuntime,
};

static RUNTIME: OnceLock<RetrofrontRuntime> = OnceLock::new();

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
    if runtime.filesystem.ensure_layout().is_err() {
        return false;
    }
    RUNTIME.set(runtime).is_ok()
}

#[no_mangle]
pub extern "C" fn retrofront_menu_api_version() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn retrofront_menu_set_title(title: *const c_char) -> bool {
    let Some(runtime) = runtime() else {
        return false;
    };
    if title.is_null() {
        return false;
    }
    let title = unsafe { CStr::from_ptr(title) }
        .to_string_lossy()
        .into_owned();
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
    if label.is_null() {
        return false;
    }
    let label = unsafe { CStr::from_ptr(label) }
        .to_string_lossy()
        .into_owned();
    let path = if path.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(path) }
            .to_string_lossy()
            .into_owned()
    };

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
    if path.is_null() {
        return false;
    }
    let path = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();
    runtime.shaders.write().set_preset(path).is_ok()
}

#[no_mangle]
pub extern "C" fn retrofront_tick() {
    if let Some(runtime) = runtime() {
        runtime.tick();
    }
}

#[no_mangle]
pub extern "C" fn retrofront_runtime_shutdown() {
    // The runtime is process-global because existing C menu callbacks do not
    // carry user data.  Resources are released by process teardown; platform
    // hosts can add explicit instance APIs later if multiple frontends are ever
    // needed in one process.
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
