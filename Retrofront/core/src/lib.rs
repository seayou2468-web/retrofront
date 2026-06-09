/// Retrofront Rust management core.

mod assets;
mod core_info;
mod dylib;
mod gfx;
mod launch;
mod libretro;
mod menu;
mod options;
mod playlist;
mod scanner;
mod settings;

use gfx::GfxRuntime;
use options::CoreOptionsManager;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::{ptr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SessionState {
    Empty = 0,
    CoreLoaded = 1,
    GameLoaded = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    pub library_name: String,
    pub library_version: String,
    pub valid_extensions: Vec<String>,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub path: String,
    pub meta: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendEvent {
    VideoFrame {
        width: u32,
        height: u32,
        pitch: usize,
        pixel_format: u32,
        frame_number: u64,
    },
    AudioBatch {
        frames: usize,
    },
    AudioSample {
        left: i16,
        right: i16,
    },
    EnvironmentCommand {
        command: u32,
        handled: bool,
    },
    RequestExtractAssets,
    InputPoll,
}

pub struct FrontendCore {
    system_info: Option<SystemInfo>,
    game: Option<GameInfo>,
    events: VecDeque<FrontendEvent>,
    joypad_buttons: [i16; 16],
    pub gfx: GfxRuntime,
    pub options: CoreOptionsManager,
    pub core_info: core_info::CoreInfoList,
    pub settings: settings::Settings,
    pub menu: menu::MenuEngine,
    pub scanner: scanner::Scanner,
    pub launcher: launch::LaunchManager,
}

static CORE_INSTANCE: OnceLock<Mutex<FrontendCore>> = OnceLock::new();

thread_local! {
    static ACTIVE_CORE: RefCell<*mut FrontendCore> = RefCell::new(ptr::null_mut());
}

pub fn with_active_frontend<F, R>(f: F) -> R
where
    F: FnOnce(&mut FrontendCore) -> R,
{
    ACTIVE_CORE.with(|active| {
        let current_ptr = *active.borrow();
        if !current_ptr.is_null() {
            return f(unsafe { &mut *current_ptr });
        }

        let mutex = CORE_INSTANCE.get_or_init(|| Mutex::new(FrontendCore::new()));
        let mut core = mutex.lock().unwrap();

        let core_ptr = &mut *core as *mut FrontendCore;
        *active.borrow_mut() = core_ptr;
        let res = f(&mut core);
        *active.borrow_mut() = ptr::null_mut();
        res
    })
}

impl FrontendCore {
    pub fn new() -> Self {
        let mut core = Self {
            system_info: None,
            game: None,
            events: VecDeque::new(),
            joypad_buttons: [0; 16],
            gfx: GfxRuntime::new(),
            options: CoreOptionsManager::new(),
            core_info: core_info::CoreInfoList::new(),
            settings: settings::Settings::new(),
            menu: menu::MenuEngine::new(),
            scanner: scanner::Scanner::new(),
            launcher: launch::LaunchManager::new(),
        };
        core.configure_from_settings();
        core
    }

    pub fn state(&self) -> SessionState {
        if self.game.is_some() { SessionState::GameLoaded }
        else if self.system_info.is_some() { SessionState::CoreLoaded }
        else { SessionState::Empty }
    }

    pub fn queue_event(&mut self, event: FrontendEvent) { self.events.push_back(event); }
    pub fn next_event(&mut self) -> Option<FrontendEvent> { self.events.pop_front() }

    pub fn configure_from_settings(&mut self) {
        self.core_info.set_info_dir(self.settings.libretro_info_path());
        self.core_info.scan_directory(&self.settings.libretro_directory());
    }

    pub fn system_info(&self) -> Option<&SystemInfo> { self.system_info.as_ref() }
    pub fn game_info(&self) -> Option<&GameInfo> { self.game.as_ref() }

    pub fn set_joypad_button(&mut self, id: u32, pressed: bool) -> Result<(), String> {
        if id < 16 { self.joypad_buttons[id as usize] = if pressed { 1 } else { 0 }; Ok(()) }
        else { Err(format!("invalid button: {}", id)) }
    }

    pub fn load_core(&mut self, _path: String) -> Result<(), String> {
        self.system_info = Some(SystemInfo {
            library_name: "Retrofront Core".to_string(),
            library_version: "1.0".to_string(),
            valid_extensions: vec!["gba".to_string()],
            need_fullpath: true,
            block_extract: false,
        });
        Ok(())
    }

    pub fn load_game(&mut self, path: String, meta: Option<String>) -> Result<(), String> {
        self.game = Some(GameInfo { path, meta: meta.unwrap_or_default() });
        Ok(())
    }

    pub fn unload_game(&mut self) { self.game = None; }
}

#[repr(C)]
pub struct RfFrontend {
    last_error: CString,
    cached_strings: Vec<CString>,
    info_name: CString,
    info_version: CString,
    info_extensions: CString,
}

#[repr(C)] pub struct RfSystemInfo { pub library_name: *const c_char, pub library_version: *const c_char, pub valid_extensions: *const c_char, pub need_fullpath: bool, pub block_extract: bool }
#[repr(C)] pub struct RfEvent { pub kind: u32, pub a: u64, pub b: u64, pub c: u64 }
#[repr(C)] pub struct RfVideoFrameInfo { pub width: u32, pub height: u32, pub pitch: u64, pub rgba_len: u64, pub pixel_format: u32, pub frame_number: u64 }
#[repr(C)] pub struct RfCoreOptionValue { pub value: *const c_char, pub label: *const c_char }
#[repr(C)] pub struct RfCoreOption { pub key: *const c_char, pub desc: *const c_char, pub info: *const c_char, pub value: *const c_char, pub values: *const RfCoreOptionValue, pub values_count: usize }
#[repr(C)] pub struct RfCoreInfo { pub path: *const c_char, pub display_name: *const c_char, pub system_name: *const c_char, pub supported_extensions: *const c_char }
#[repr(C)] pub struct RfGameEntry { pub path: *const c_char, pub label: *const c_char }
#[repr(C)] pub struct RfMenuList { pub title: *const c_char, pub entry_count: usize }
#[repr(C)] pub struct RfMenuEntry { pub label: *const c_char, pub sublabel: *const c_char, pub kind: u32, pub value: *const c_char, pub action_id: u32 }
#[repr(C)] pub struct RfSettingEntry { pub key: *const c_char, pub value: *const c_char }
#[repr(C)] pub struct RfLaunchPlan { pub content_path: *const c_char, pub content_extension: *const c_char, pub decision: u32, pub selected_core_path: *const c_char, pub candidate_count: usize, pub reason: *const c_char }
#[repr(C)] pub struct RfGfxVideoConfig { pub base_width: u32, pub base_height: u32, pub max_width: u32, pub max_height: u32, pub aspect_ratio: f32, pub output_width: u32, pub output_height: u32, pub scale_mode: u32, pub filter_mode: u32, pub rotation_quarters: u32, pub vsync: bool }
#[repr(C)] pub struct RfGfxHostHandles { pub native_view: u64, pub context: u64, pub framebuffer: usize, pub render_callback: *const std::ffi::c_void, pub get_proc_address: *const std::ffi::c_void, pub user_data: *mut std::ffi::c_void }
#[repr(C)] pub struct RfGfxDriverInfo { pub backend: u32, pub frame_number: u64, pub hardware_ready: bool, pub rendered: bool }

#[no_mangle] pub unsafe extern "C" fn rf_frontend_create() -> *mut RfFrontend { Box::into_raw(Box::new(RfFrontend { last_error: CString::default(), cached_strings: Vec::new(), info_name: CString::default(), info_version: CString::default(), info_extensions: CString::default() })) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_destroy(f: *mut RfFrontend) { if !f.is_null() { unsafe { drop(Box::from_raw(f)); } } }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_state(_f: *const RfFrontend) -> u32 { with_active_frontend(|c| c.state() as u32) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_load_core(f: *mut RfFrontend, path: *const c_char) -> bool { let f = unsafe { f.as_mut() }.unwrap(); let ps = ptr_to_str(path).unwrap(); let res = with_active_frontend(|c| c.load_core(ps)); match res { Ok(()) => { f.last_error = CString::default(); if let Some(info) = with_active_frontend(|c| c.system_info().cloned()) { cache_system_info(f, &info); } true } Err(e) => { set_error(f, &e); false } } }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_load_game(f: *mut RfFrontend, path: *const c_char, meta: *const c_char) -> bool { let f = unsafe { f.as_mut() }.unwrap(); let ps = ptr_to_str(path).unwrap(); let ms = ptr_to_str(meta); let res = with_active_frontend(|c| c.load_game(ps, ms)); match res { Ok(()) => { f.last_error = CString::default(); true } Err(e) => { set_error(f, &e); false } } }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_launch_content(f: *mut RfFrontend, p: *const c_char, cp: *const c_char, m: *const c_char) -> bool { if !unsafe { rf_frontend_load_core(f, cp) } { return false; } unsafe { rf_frontend_load_game(f, p, m) } }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_run_frame(_f: *mut RfFrontend) -> bool { true }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_unload_game(_f: *mut RfFrontend) { with_active_frontend(|c| c.unload_game()); }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_last_error(f: *const RfFrontend) -> *const c_char { unsafe { f.as_ref() }.map_or(ptr::null(), |f| f.last_error.as_ptr()) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_next_event(_f: *mut RfFrontend, out: *mut RfEvent) -> bool { let out = unsafe { out.as_mut() }.unwrap(); with_active_frontend(|c| { if let Some(e) = c.next_event() { match e { FrontendEvent::VideoFrame { width, height, pitch, .. } => { out.kind = 1; out.a = width as u64; out.b = height as u64; out.c = pitch as u64; } FrontendEvent::AudioBatch { frames } => { out.kind = 2; out.a = frames as u64; } FrontendEvent::AudioSample { left, right } => { out.kind = 3; out.a = left as i16 as u64; out.b = right as i16 as u64; } FrontendEvent::EnvironmentCommand { command, handled } => { out.kind = 4; out.a = command as u64; out.b = handled as u64; } FrontendEvent::InputPoll => { out.kind = 5; } FrontendEvent::RequestExtractAssets => { out.kind = 6; } } true } else { false } }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_menu_current_list(f: *mut RfFrontend, out: *mut RfMenuList) -> bool { let f = unsafe { f.as_mut() }.unwrap(); let out = unsafe { out.as_mut() }.unwrap(); with_active_frontend(|c| { if let Some(cur) = c.menu.current() { let tc = CString::new(cur.title.clone()).unwrap_or_default(); out.title = tc.as_ptr(); f.cached_strings.push(tc); out.entry_count = cur.entries.len(); true } else { false } }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_menu_get_entry(f: *mut RfFrontend, idx: usize, out: *mut RfMenuEntry) -> bool { let f = unsafe { f.as_mut() }.unwrap(); let out = unsafe { out.as_mut() }.unwrap(); with_active_frontend(|c| { if let Some(cur) = c.menu.current() { if let Some(e) = cur.entries.get(idx) { let lc = CString::new(e.label.clone()).unwrap_or_default(); let sc = CString::new(e.sublabel.clone()).unwrap_or_default(); let vc = CString::new(e.value.clone()).unwrap_or_default(); out.label = lc.as_ptr(); out.sublabel = sc.as_ptr(); out.kind = e.kind as u32; out.value = vc.as_ptr(); out.action_id = e.action_id; f.cached_strings.push(lc); f.cached_strings.push(sc); f.cached_strings.push(vc); return true; } } false }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_menu_activate(_f: *mut RfFrontend, action_id: u32) -> bool { with_active_frontend(|c| match action_id { menu::ACTION_LOAD_CONTENT => { c.menu.push_content_list(&c.scanner.games); true } menu::ACTION_QUICK_MENU => { c.menu.push_quick_menu(c.game_info().is_some()); true } menu::ACTION_EXTRACT_ASSETS => { let zp = c.settings.base_dir.join("assets.zip"); if zp.exists() { if let Err(e) = assets::extract_assets_zip(&zp, &c.settings.base_dir) { c.menu.push_status("Extraction Failed", &e); } else { c.menu.push_status("Success", "Assets extracted successfully."); } } else { c.menu.push_status("Error", "assets.zip not found."); } true } menu::ACTION_SETTINGS => { c.menu.push_settings(&c.settings); true } menu::ACTION_SETTINGS_DRIVERS => { c.menu.push_driver_settings(&c.settings); true } menu::ACTION_SETTINGS_VIDEO => { c.menu.push_video_settings(&c.settings); true } menu::ACTION_SETTINGS_AUDIO => { c.menu.push_audio_settings(&c.settings); true } menu::ACTION_SETTINGS_INPUT => { c.menu.push_input_settings(&c.settings); true } menu::ACTION_SETTINGS_DIRECTORIES => { c.menu.push_directory_settings(&c.settings); true } menu::ACTION_SKIN_SETTINGS => { c.menu.push_skin_settings(&c.settings); true } _ => false }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_menu_pop(_f: *mut RfFrontend) -> bool { with_active_frontend(|c| c.menu.pop().is_some()) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_load_settings(_f: *mut RfFrontend, p: *const c_char) -> bool { let ps = ptr_to_str(p).unwrap(); with_active_frontend(|c| { c.settings.load(Path::new(&ps)); c.configure_from_settings(); }); true }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_set_base_dir(_f: *mut RfFrontend, p: *const c_char) -> bool { let ps = ptr_to_str(p).unwrap(); with_active_frontend(|c| { c.settings.set_base_dir(Path::new(&ps)); c.configure_from_settings(); }); true }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_save_settings(_f: *mut RfFrontend) { with_active_frontend(|c| c.settings.save()); }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_get_setting(f: *mut RfFrontend, k: *const c_char) -> *const c_char { let f = unsafe { f.as_mut() }.unwrap(); let ks = ptr_to_str(k).unwrap(); with_active_frontend(|c| { let v = c.settings.get(&ks).cloned().unwrap_or_default(); let vc = CString::new(v).unwrap_or_default(); let p = vc.as_ptr(); f.cached_strings.push(vc); p }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_set_setting(_f: *mut RfFrontend, k: *const c_char, v: *const c_char) -> bool { let ks = ptr_to_str(k).unwrap(); let vs = ptr_to_str(v).unwrap(); with_active_frontend(|c| { c.settings.set(&ks, &vs); c.configure_from_settings(); }); true }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_settings_count(_f: *const RfFrontend) -> usize { with_active_frontend(|c| c.settings.values.len()) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_get_setting_at(f: *mut RfFrontend, idx: usize, out: *mut RfSettingEntry) -> bool { let f = unsafe { f.as_mut() }.unwrap(); let o = unsafe { out.as_mut() }.unwrap(); with_active_frontend(|c| { let mut e_list: Vec<_> = c.settings.values.iter().collect(); e_list.sort_by(|a, b| a.0.cmp(b.0)); let Some((k, v)) = e_list.get(idx) else { return false; }; let kc = CString::new(k.as_str()).unwrap_or_default(); let vc = CString::new(v.as_str()).unwrap_or_default(); o.key = kc.as_ptr(); o.value = vc.as_ptr(); f.cached_strings.push(kc); f.cached_strings.push(vc); true }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_system_info(f: *mut RfFrontend, out: *mut RfSystemInfo) -> bool { let f = unsafe { f.as_mut() }.unwrap(); let o = unsafe { out.as_mut() }.unwrap(); with_active_frontend(|c| { if let Some(i) = c.system_info() { let nc = CString::new(i.library_name.as_str()).unwrap_or_default(); let vc = CString::new(i.library_version.as_str()).unwrap_or_default(); let ec = CString::new(i.valid_extensions.join("|")).unwrap_or_default(); o.library_name = nc.as_ptr(); o.library_version = vc.as_ptr(); o.valid_extensions = ec.as_ptr(); o.need_fullpath = i.need_fullpath; o.block_extract = i.block_extract; f.cached_strings.push(nc); f.cached_strings.push(vc); f.cached_strings.push(ec); true } else { false } }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_plan_content_launch(_f: *mut RfFrontend, p: *const c_char, pc: *const c_char, out: *mut RfLaunchPlan) -> bool { let o = unsafe { out.as_mut() }.unwrap(); let ps = ptr_to_str(p).unwrap_or_default(); let pcs = ptr_to_str(pc); let pp = pcs.map(PathBuf::from); with_active_frontend(|c| { let p = c.launcher.plan_content_launch(Path::new(&ps), &c.core_info, &c.settings, pp.as_deref()); o.content_path = CString::new(p.content_path.to_string_lossy().to_string()).unwrap().into_raw(); o.content_extension = CString::new(p.content_extension).unwrap().into_raw(); o.decision = p.decision as u32; o.selected_core_path = CString::new(p.selected_core.map(|p| p.to_string_lossy().to_string()).unwrap_or_default()).unwrap().into_raw(); o.candidate_count = p.candidates.len(); o.reason = CString::new(p.reason).unwrap().into_raw(); true }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_launch_candidate_count(_f: *const RfFrontend) -> usize { with_active_frontend(|c| c.launcher.last_plan.as_ref().map(|p| p.candidates.len()).unwrap_or(0)) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_get_launch_candidate(_f: *mut RfFrontend, idx: usize, out: *mut RfCoreInfo) -> bool { let o = unsafe { out.as_mut() }.unwrap(); with_active_frontend(|c| { if let Some(p) = c.launcher.last_plan.as_ref() { if let Some(can) = p.candidates.get(idx) { o.path = CString::new(can.path.to_string_lossy().to_string()).unwrap().into_raw(); o.display_name = CString::new(can.display_name.clone()).unwrap().into_raw(); o.system_name = CString::new(can.system_name.clone()).unwrap().into_raw(); o.supported_extensions = CString::new(can.supported_extensions.join("|")).unwrap().into_raw(); return true; } } false }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_scan_configured_cores(_f: *mut RfFrontend) { with_active_frontend(|c| c.configure_from_settings()); }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_all_extensions(_f: *mut RfFrontend) -> *const c_char { with_active_frontend(|c| { let e = c.core_info.all_extensions.join("|"); CString::new(e).unwrap().into_raw() }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_cores_count(_f: *const RfFrontend) -> usize { with_active_frontend(|c| c.core_info.cores.len()) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_get_core_info(_f: *mut RfFrontend, idx: usize, out: *mut RfCoreInfo) -> bool { let o = unsafe { out.as_mut() }.unwrap(); with_active_frontend(|c| { if let Some(can) = c.core_info.cores.get(idx) { o.path = CString::new(can.path.to_string_lossy().to_string()).unwrap().into_raw(); o.display_name = CString::new(can.display_name.clone()).unwrap().into_raw(); o.system_name = CString::new(can.system_name.clone()).unwrap().into_raw(); o.supported_extensions = CString::new(can.supported_extensions.join("|")).unwrap().into_raw(); return true; } false }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_games_count(_f: *const RfFrontend) -> usize { with_active_frontend(|c| c.scanner.games.len()) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_get_game_info(_f: *mut RfFrontend, idx: usize, out: *mut RfGameEntry) -> bool { let o = unsafe { out.as_mut() }.unwrap(); with_active_frontend(|c| { if let Some(g) = c.scanner.games.get(idx) { o.path = CString::new(g.path.to_string_lossy().to_string()).unwrap().into_raw(); o.label = CString::new(g.label.clone()).unwrap().into_raw(); return true; } false }) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_scan_games(_f: *mut RfFrontend, d: *const c_char, e: *const c_char) { let ds = ptr_to_str(d).unwrap_or_default(); let es = ptr_to_str(e).unwrap_or_default(); with_active_frontend(|c| c.scanner.scan_directory(Path::new(&ds), &es)); }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_set_info_dir(_f: *mut RfFrontend, p: *const c_char) { let ps = ptr_to_str(p).unwrap_or_default(); with_active_frontend(|c| c.core_info.set_info_dir(PathBuf::from(ps))); }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_scan_cores(_f: *mut RfFrontend, p: *const c_char) { let ps = ptr_to_str(p).unwrap_or_default(); with_active_frontend(|c| c.core_info.scan_directory(Path::new(&ps))); }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_set_gfx_backend(_f: *mut RfFrontend, _b: u32) -> bool { true }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_set_joypad_button(_f: *mut RfFrontend, id: u32, p: bool) -> bool { with_active_frontend(|c| c.set_joypad_button(id, p).is_ok()) }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_video_frame_info(_f: *const RfFrontend, _o: *mut RfVideoFrameInfo) -> bool { false }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_copy_video_frame_rgba(_f: *const RfFrontend, _o: *mut u8, _l: usize) -> usize { 0 }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_get_gfx_video_config(_f: *const RfFrontend, _o: *mut RfGfxVideoConfig) -> bool { false }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_set_gfx_video_config(_f: *mut RfFrontend, _c: *const RfGfxVideoConfig) -> bool { true }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_set_gfx_host_handles(_f: *mut RfFrontend, _h: *const RfGfxHostHandles) -> bool { true }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_gfx_driver_info(_f: *const RfFrontend, _o: *mut RfGfxDriverInfo) -> bool { false }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_set_options_config_path(_f: *mut RfFrontend, _p: *const c_char) -> bool { true }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_options_count(_f: *const RfFrontend) -> usize { 0 }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_get_option(_f: *mut RfFrontend, _i: usize, _o: *mut RfCoreOption) -> bool { false }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_set_option(_f: *mut RfFrontend, _k: *const c_char, _v: *const c_char) -> bool { true }
#[no_mangle] pub unsafe extern "C" fn rf_frontend_clear_options_cache(_f: *mut RfFrontend) {}
#[no_mangle] pub unsafe extern "C" fn rf_frontend_menu_push_core_list(_f: *mut RfFrontend) {}
#[no_mangle] pub unsafe extern "C" fn rf_frontend_menu_push_content_list(_f: *mut RfFrontend) {}
#[no_mangle] pub unsafe extern "C" fn rf_frontend_menu_push_information(_f: *mut RfFrontend) {}
#[no_mangle] pub unsafe extern "C" fn rf_frontend_menu_push_skin_settings(f: *mut RfFrontend) { with_active_frontend(|c| c.menu.push_skin_settings(&c.settings)); }

fn ptr_to_str(ptr: *const c_char) -> Option<String> { if ptr.is_null() { None } else { Some(unsafe { CStr::from_ptr(ptr) }.to_str().ok()?.to_owned()) } }
fn set_error(f: &mut RfFrontend, m: &str) { f.last_error = CString::new(m).unwrap_or_else(|_| CString::new("error").unwrap()); }
fn cache_system_info(f: &mut RfFrontend, i: &SystemInfo) { f.info_name = CString::new(i.library_name.as_str()).unwrap_or_default(); f.info_version = CString::new(i.library_version.as_str()).unwrap_or_default(); f.info_extensions = CString::new(i.valid_extensions.join("|")).unwrap_or_default(); }
