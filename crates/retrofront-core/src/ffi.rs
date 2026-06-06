use crate::{FrontendConfig, RetroHost};
use std::{
    ffi::{c_char, CStr, CString},
    path::Path,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RfFrameInfo {
    pub width: u32,
    pub height: u32,
    pub pitch: usize,
    pub video_bytes: usize,
    pub audio_frames: usize,
}

pub struct RfHost {
    host: RetroHost,
    last_error: Option<CString>,
}

#[no_mangle]
pub unsafe extern "C" fn rf_host_create(
    core_path: *const c_char,
    config_path: *const c_char,
) -> *mut RfHost {
    let result = (|| {
        let core = c_path(core_path)?;
        let config = if config_path.is_null() {
            FrontendConfig::default()
        } else {
            FrontendConfig::load_retroarch_cfg(Path::new(&c_path(config_path)?))?
        };
        RetroHost::load_core(core, config)
    })();
    match result {
        Ok(host) => Box::into_raw(Box::new(RfHost {
            host,
            last_error: None,
        })),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_host_destroy(host: *mut RfHost) {
    if !host.is_null() {
        drop(Box::from_raw(host));
    }
}

#[no_mangle]
pub unsafe extern "C" fn rf_host_load_game(host: *mut RfHost, content_path: *const c_char) -> bool {
    with_host(host, |h| {
        let content = if content_path.is_null() {
            None
        } else {
            Some(c_path(content_path)?)
        };
        h.host.load_game(content.as_deref().map(Path::new))
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_host_run_frame(host: *mut RfHost, out: *mut RfFrameInfo) -> bool {
    with_host(host, |h| {
        let frame = h.host.run_frame()?;
        if !out.is_null() {
            *out = RfFrameInfo {
                width: frame.width,
                height: frame.height,
                pitch: frame.pitch,
                video_bytes: frame.video_bytes,
                audio_frames: frame.audio_frames,
            };
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn rf_host_set_joypad_button(
    host: *mut RfHost,
    port: u32,
    id: u32,
    pressed: bool,
) -> bool {
    with_host(host, |h| h.host.set_joypad_button(port, id, pressed))
}

#[no_mangle]
pub unsafe extern "C" fn rf_host_clear_input(host: *mut RfHost) -> bool {
    with_host(host, |h| h.host.clear_input())
}

#[no_mangle]
pub unsafe extern "C" fn rf_host_last_error(host: *mut RfHost) -> *const c_char {
    host.as_mut()
        .and_then(|h| h.last_error.as_ref())
        .map_or(std::ptr::null(), |e| e.as_ptr())
}

unsafe fn with_host<F>(host: *mut RfHost, f: F) -> bool
where
    F: FnOnce(&mut RfHost) -> Result<(), String>,
{
    let Some(h) = host.as_mut() else {
        return false;
    };
    match f(h) {
        Ok(()) => {
            h.last_error = None;
            true
        }
        Err(e) => {
            h.last_error = CString::new(e).ok();
            false
        }
    }
}

unsafe fn c_path(pointer: *const c_char) -> Result<String, String> {
    if pointer.is_null() {
        return Err("null path".into());
    }
    Ok(CStr::from_ptr(pointer).to_string_lossy().into_owned())
}
