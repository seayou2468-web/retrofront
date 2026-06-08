use super::hardware::{GfxBackendKind, HardwareRenderRequest, HostRenderHandles};
use crate::libretro;
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextEvent {
    Reset,
    Destroyed,
}

#[derive(Debug, Clone, Default)]
pub struct ContextDriver {
    request: Option<HardwareRenderRequest>,
    handles: HostRenderHandles,
    context_reset: libretro::retro_hw_context_reset_t,
    context_destroy: libretro::retro_hw_context_reset_t,
    reset_count: u64,
    destroy_count: u64,
}

impl ContextDriver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hardware_request(&self) -> Option<HardwareRenderRequest> {
        self.request
    }

    pub fn handles(&self) -> HostRenderHandles {
        self.handles
    }

    pub fn set_hardware_request(&mut self, request: HardwareRenderRequest) {
        self.request = Some(request);
    }

    pub fn capture_callbacks(&mut self, raw: &libretro::retro_hw_render_callback) {
        self.context_reset = raw.context_reset;
        self.context_destroy = raw.context_destroy;
    }

    pub fn clear_request(&mut self) {
        self.notify_destroy();
        self.request = None;
    }

    pub fn set_handles(&mut self, handles: HostRenderHandles) {
        self.handles = handles;
    }

    pub fn hardware_ready(&self) -> bool {
        self.request
            .map(|request| match request.preferred_backend() {
                GfxBackendKind::Software => true,
                GfxBackendKind::Bgfx => self.handles.is_valid(),
            })
            .unwrap_or(false)
    }

    pub fn apply_event(&mut self, event: ContextEvent) {
        match event {
            ContextEvent::Reset => self.notify_reset(),
            ContextEvent::Destroyed => self.notify_destroy(),
        }
    }

    pub fn notify_reset(&mut self) {
        self.reset_count = self.reset_count.saturating_add(1);
        if let Some(callback) = self.context_reset {
            unsafe { callback() };
        }
    }

    pub fn notify_destroy(&mut self) {
        self.destroy_count = self.destroy_count.saturating_add(1);
        if let Some(callback) = self.context_destroy {
            unsafe { callback() };
        }
    }

    pub fn patch_hw_render_callback(&self, raw: &mut libretro::retro_hw_render_callback) {
        raw.get_current_framebuffer = Some(get_current_framebuffer);
        raw.get_proc_address = Some(get_proc_address);
    }
}

unsafe extern "C" fn get_current_framebuffer() -> usize {
    crate::with_active_frontend(|frontend| frontend.gfx.context_handles().framebuffer)
}

unsafe extern "C" fn get_proc_address(sym: *const c_char) -> libretro::retro_proc_address_t {
    if sym.is_null() {
        return None;
    }
    let symbol = unsafe { CStr::from_ptr(sym) }.to_bytes();
    if symbol.is_empty() {
        return None;
    }
    crate::with_active_frontend(|frontend| {
        let handles = frontend.gfx.context_handles();
        handles
            .get_proc_address
            .and_then(|callback| unsafe { callback(sym, handles.user_data) })
    })
}
