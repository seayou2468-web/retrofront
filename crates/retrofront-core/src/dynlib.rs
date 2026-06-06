use std::ffi::{c_char, c_int, c_void, CString};
use std::path::Path;

#[cfg(unix)]
#[link(name = "dl")]
extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
}

#[cfg(unix)]
const RTLD_NOW: c_int = 2;

#[derive(Debug)]
pub struct DynamicLibrary {
    handle: *mut c_void,
}

unsafe impl Send for DynamicLibrary {}
unsafe impl Sync for DynamicLibrary {}

impl DynamicLibrary {
    pub fn open(path: &Path) -> Result<Self, String> {
        #[cfg(unix)]
        unsafe {
            let path =
                CString::new(path.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
            let handle = dlopen(path.as_ptr(), RTLD_NOW);
            if handle.is_null() {
                return Err(last_error());
            }
            Ok(Self { handle })
        }
        #[cfg(not(unix))]
        {
            let _ = path;
            Err("dynamic libretro loading is implemented for Unix targets".into())
        }
    }

    pub unsafe fn symbol<T: Copy>(&self, name: &str) -> Result<T, String> {
        #[cfg(unix)]
        {
            let name = CString::new(name).map_err(|e| e.to_string())?;
            let pointer = dlsym(self.handle, name.as_ptr());
            if pointer.is_null() {
                Err(last_error())
            } else {
                Ok(std::mem::transmute_copy(&pointer))
            }
        }
        #[cfg(not(unix))]
        {
            let _ = name;
            Err("dynamic symbol loading is implemented for Unix targets".into())
        }
    }
}

impl Drop for DynamicLibrary {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            if !self.handle.is_null() {
                let _ = dlclose(self.handle);
            }
        }
    }
}

#[cfg(unix)]
unsafe fn last_error() -> String {
    let err = dlerror();
    if err.is_null() {
        "unknown dlopen/dlsym error".into()
    } else {
        std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned()
    }
}
