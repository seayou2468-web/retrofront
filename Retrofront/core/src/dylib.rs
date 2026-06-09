use std::ffi::{CStr, CString};
use std::fs;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};

#[cfg(any(target_os = "linux", target_os = "android"))]
#[link(name = "dl")]
extern "C" {}

#[cfg(any(target_os = "linux", target_os = "android"))]
const RTLD_LAZY: c_int = 1;
#[cfg(any(target_os = "linux", target_os = "android"))]
const RTLD_LOCAL: c_int = 0;
#[cfg(any(target_os = "macos", target_os = "ios"))]
const RTLD_LAZY: c_int = 1;
#[cfg(any(target_os = "macos", target_os = "ios"))]
const RTLD_LOCAL: c_int = 4;

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
        let load_path = library_load_path(path);
        let c_path = CString::new(load_path.as_os_str().to_string_lossy().as_bytes())
            .map_err(|_| format!("path contains an interior NUL: {}", load_path.display()))?;
        let handle = unsafe { dlopen(c_path.as_ptr(), RTLD_LAZY | RTLD_LOCAL) };
        if handle.is_null() {
            return Err(last_dl_error()
                .unwrap_or_else(|| format!("failed to open {}", load_path.display())));
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

fn library_load_path(path: &Path) -> PathBuf {
    if is_framework_dir(path) {
        if let Some(binary_path) = framework_binary_path(path) {
            return binary_path;
        }
    }
    path.to_path_buf()
}

fn is_framework_dir(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("framework"))
}

fn framework_binary_path(path: &Path) -> Option<PathBuf> {
    for name in framework_executable_candidates(path) {
        let candidate = path.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    let stem = path.file_stem()?.to_str()?;
    Some(path.join(stem))
}

fn framework_executable_candidates(path: &Path) -> Vec<String> {
    let mut candidates = Vec::new();
    if let Some(executable) = framework_info_plist_executable(path) {
        push_unique(&mut candidates, executable);
    }
    if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
        push_unique(&mut candidates, stem.to_string());
        if let Some(stripped) = stem.strip_suffix(".libretro") {
            push_unique(&mut candidates, stripped.to_string());
        }
        if let Some(stripped) = stem.strip_suffix("_libretro") {
            push_unique(&mut candidates, stripped.to_string());
        }
    }
    candidates
}

fn framework_info_plist_executable(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path.join("Info.plist")).ok()?;
    let key_pos = content.find("<key>CFBundleExecutable</key>")?;
    let after_key = &content[key_pos..];
    let string_start = after_key.find("<string>")? + "<string>".len();
    let after_start = &after_key[string_start..];
    let string_end = after_start.find("</string>")?;
    let value = decode_minimal_xml_entities(after_start[..string_end].trim());
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn decode_minimal_xml_entities(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.iter().any(|existing| existing == &value) {
        values.push(value);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "retrofront-dylib-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn resolves_framework_bundle_to_inner_binary() {
        assert_eq!(
            library_load_path(Path::new("/cores/mgba_libretro_ios.framework")),
            PathBuf::from("/cores/mgba_libretro_ios.framework/mgba_libretro_ios")
        );
    }

    #[test]
    fn resolves_dotted_framework_bundle_to_dotted_inner_binary() {
        assert_eq!(
            library_load_path(Path::new("/cores/azahar.libretro.framework")),
            PathBuf::from("/cores/azahar.libretro.framework/azahar.libretro")
        );
    }

    #[test]
    fn resolves_framework_bundle_using_info_plist_executable() {
        let dir = temp_dir("plist-executable");
        let framework = dir.join("Example.libretro.framework");
        fs::create_dir_all(&framework).unwrap();
        fs::write(
            framework.join("Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>ExampleCore</string>
</dict></plist>"#,
        )
        .unwrap();
        fs::write(framework.join("ExampleCore"), "").unwrap();

        assert_eq!(library_load_path(&framework), framework.join("ExampleCore"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn falls_back_to_existing_stripped_libretro_framework_binary() {
        let dir = temp_dir("stripped-libretro");
        let framework = dir.join("Example.libretro.framework");
        fs::create_dir_all(&framework).unwrap();
        fs::write(framework.join("Example"), "").unwrap();

        assert_eq!(library_load_path(&framework), framework.join("Example"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn leaves_plain_dynamic_library_paths_unchanged() {
        assert_eq!(
            library_load_path(Path::new("/cores/mgba_libretro_ios.dylib")),
            PathBuf::from("/cores/mgba_libretro_ios.dylib")
        );
    }
}
