use super::hardware::{HardwareRenderRequest, HostRenderHandles};
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
    reset_count: u64,
    destroy_count: u64,
}

impl ContextDriver {
    pub fn request(&self) -> Option<HardwareRenderRequest> {
        self.request
    }
    pub fn handles(&self) -> HostRenderHandles {
        self.handles
    }
    pub fn set_request(&mut self, request: HardwareRenderRequest) {
        self.request = Some(request);
    }
    pub fn clear_request(&mut self) {
        self.request = None;
    }
    pub fn set_host_handles(&mut self, handles: HostRenderHandles) {
        self.handles = handles;
    }

    pub fn hardware_ready(&self) -> bool {
        self.request
            .map(|request| match request.preferred_backend() {
                super::hardware::GfxBackendKind::Software => true,
                super::hardware::GfxBackendKind::OpenGl => self.handles.has_opengl(),
                super::hardware::GfxBackendKind::Vulkan => self.handles.has_vulkan(),
            })
            .unwrap_or(false)
    }

    pub fn apply_event(&mut self, event: ContextEvent) {
        match event {
            ContextEvent::Reset => self.reset_count = self.reset_count.saturating_add(1),
            ContextEvent::Destroyed => self.destroy_count = self.destroy_count.saturating_add(1),
        }
    }

    pub fn patch_hw_render_callback(&self, raw: &mut libretro::retro_hw_render_callback) {
        raw.get_current_framebuffer = Some(get_current_framebuffer);
        raw.get_proc_address = Some(get_proc_address);
    }
}

unsafe extern "C" fn get_current_framebuffer() -> usize {
    usize::MAX
}

unsafe extern "C" fn get_proc_address(sym: *const c_char) -> libretro::retro_proc_address_t {
    if sym.is_null() {
        return None;
    }
    let symbol = unsafe { CStr::from_ptr(sym) }.to_bytes();
    if symbol.is_empty() {
        None
    } else {
        None
    }
}
