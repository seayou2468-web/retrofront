#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code
)]

/// Raw bindgen output from `Retrofront/frontend/libretro/libretro.h`.
pub mod sys {
    include!(concat!(env!("OUT_DIR"), "/libretro_bindings.rs"));
}

use std::ffi::{CStr, CString};

/// Safe Rust representation of `retro_system_info`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemInfo {
    pub library_name: String,
    pub library_version: String,
    pub valid_extensions: Vec<String>,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

impl SystemInfo {
    /// Convert bindgen's raw C struct into owned Rust values.
    ///
    /// The wrapper exists because the raw callback ABI exposes nullable C
    /// strings, booleans and lifetimes that are unsafe to use directly from menu
    /// code.
    pub unsafe fn from_raw(raw: &sys::retro_system_info) -> Self {
        fn opt_string(ptr: *const std::os::raw::c_char) -> String {
            if ptr.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned()
            }
        }

        let valid_extensions = opt_string(raw.valid_extensions)
            .split('|')
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        Self {
            library_name: opt_string(raw.library_name),
            library_version: opt_string(raw.library_version),
            valid_extensions,
            need_fullpath: raw.need_fullpath,
            block_extract: raw.block_extract,
        }
    }
}

/// Owned content descriptor passed to a libretro core.
#[derive(Clone, Debug, Default)]
pub struct GameInfo {
    pub path: Option<std::path::PathBuf>,
    pub data: Option<Vec<u8>>,
    pub meta: Option<String>,
}

/// Pins C strings and byte buffers while a `retro_game_info` pointer is in use.
pub struct GameInfoHandle {
    path: Option<CString>,
    meta: Option<CString>,
    data: Option<Vec<u8>>,
    raw: sys::retro_game_info,
}

impl GameInfoHandle {
    pub fn new(info: GameInfo) -> Result<Self, std::ffi::NulError> {
        let path = info
            .path
            .map(|p| CString::new(p.to_string_lossy().as_bytes()))
            .transpose()?;
        let meta = info.meta.map(CString::new).transpose()?;
        let data = info.data;

        let raw = sys::retro_game_info {
            path: path.as_ref().map_or(std::ptr::null(), |p| p.as_ptr()),
            data: data
                .as_ref()
                .map_or(std::ptr::null(), |bytes| bytes.as_ptr().cast()),
            size: data.as_ref().map_or(0, Vec::len),
            meta: meta.as_ref().map_or(std::ptr::null(), |m| m.as_ptr()),
        };

        Ok(Self {
            path,
            meta,
            data,
            raw,
        })
    }

    pub fn as_ptr(&self) -> *const sys::retro_game_info {
        &self.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_info_handle_preserves_owned_data() {
        let handle = GameInfoHandle::new(GameInfo {
            path: Some("roms/demo.gba".into()),
            data: Some(vec![1, 2, 3]),
            meta: Some("demo".into()),
        })
        .unwrap();
        assert!(!handle.raw.path.is_null());
        assert_eq!(handle.raw.size, 3);
        assert!(!handle.raw.meta.is_null());
    }
}
