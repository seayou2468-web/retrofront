use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};

#[cfg(any(target_os = "linux", target_os = "android"))]
#[link(name = "dl")]
extern "C" {}

#[cfg(any(target_os = "linux", target_os = "android"))]
const RTLD_NOW: c_int = 2;
#[cfg(any(target_os = "macos", target_os = "ios"))]
const RTLD_NOW: c_int = 2;

extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
}

#[derive(Debug)]
pub struct Library {
    handle: *mut c_void,
    path: PathBuf,
}

unsafe impl Send for Library {}
unsafe impl Sync for Library {}

#[derive(Clone, Copy)]
pub struct Symbol<T> {
    ptr: T,
    _marker: PhantomData<T>,
}

impl<T: Copy> Symbol<T> {
    pub fn get(self) -> T {
        self.ptr
    }
}

impl Library {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let c_path = CString::new(path.as_os_str().to_string_lossy().as_bytes())
            .map_err(|_| format!("path contains an interior NUL: {}", path.display()))?;
        let handle = unsafe { dlopen(c_path.as_ptr(), RTLD_NOW) };
        if handle.is_null() {
            return Err(
                last_dl_error().unwrap_or_else(|| format!("failed to open {}", path.display()))
            );
        }
        Ok(Self {
            handle,
            path: path.to_path_buf(),
        })
    }

    pub fn symbol<T: Copy>(&self, name: &CStr) -> Result<Symbol<T>, String> {
        let ptr = unsafe { dlsym(self.handle, name.as_ptr()) };
        if ptr.is_null() {
            return Err(last_dl_error().unwrap_or_else(|| {
                format!(
                    "missing symbol {} in {}",
                    name.to_string_lossy(),
                    self.path.display()
                )
            }));
        }
        let typed = unsafe { std::mem::transmute_copy::<*mut c_void, T>(&ptr) };
        Ok(Symbol {
            ptr: typed,
            _marker: PhantomData,
        })
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            let _ = unsafe { dlclose(self.handle) };
            self.handle = std::ptr::null_mut();
        }
    }
}

fn last_dl_error() -> Option<String> {
    let err = unsafe { dlerror() };
    if err.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(err) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}
