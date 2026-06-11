use crate::libretro;
use std::os::raw::{c_char, c_void};

/// Rendering backend requested by the host application or libretro core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GfxBackendKind {
    /// CPU-backed path. Always available and used as compatibility fallback.
    Software = 0,
    /// Automatic wgpu path. The host chooses the best available adapter/backend.
    Wgpu = 1,
    /// wgpu using Metal where available.
    Metal = 2,
    /// wgpu using OpenGL/OpenGL ES where available.
    OpenGl = 3,
    /// wgpu using Vulkan where available.
    Vulkan = 4,
    /// wgpu using Vulkan over MoltenVK on Apple platforms.
    MoltenVk = 5,
}

impl GfxBackendKind {
    pub fn is_wgpu_family(self) -> bool {
        matches!(
            self,
            Self::Wgpu | Self::Metal | Self::OpenGl | Self::Vulkan | Self::MoltenVk
        )
    }
}

/// Information captured from `RETRO_ENVIRONMENT_SET_HW_RENDER`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareRenderRequest {
    pub context_type: u32,
    pub version_major: u32,
    pub version_minor: u32,
    pub depth: bool,
    pub stencil: bool,
    pub bottom_left_origin: bool,
    pub cache_context: bool,
    pub debug_context: bool,
}

impl HardwareRenderRequest {
    pub fn from_libretro(raw: &libretro::retro_hw_render_callback) -> Self {
        Self {
            context_type: raw.context_type,
            version_major: raw.version_major,
            version_minor: raw.version_minor,
            depth: raw.depth,
            stencil: raw.stencil,
            bottom_left_origin: raw.bottom_left_origin,
            cache_context: raw.cache_context,
            debug_context: raw.debug_context,
        }
    }

    pub fn opengles(major: u32, minor: u32) -> Self {
        Self {
            context_type: libretro::retro_hw_context_type_RETRO_HW_CONTEXT_OPENGLES_VERSION,
            version_major: major,
            version_minor: minor,
            depth: false,
            stencil: false,
            bottom_left_origin: false,
            cache_context: true,
            debug_context: false,
        }
    }

    pub fn vulkan() -> Self {
        Self {
            context_type: libretro::retro_hw_context_type_RETRO_HW_CONTEXT_VULKAN,
            version_major: 1,
            version_minor: 0,
            depth: false,
            stencil: false,
            bottom_left_origin: false,
            cache_context: true,
            debug_context: false,
        }
    }

    pub fn preferred_backend(self) -> GfxBackendKind {
        match self.context_type {
            value if value == libretro::retro_hw_context_type_RETRO_HW_CONTEXT_VULKAN => {
                GfxBackendKind::Vulkan
            }
            value
                if value == libretro::retro_hw_context_type_RETRO_HW_CONTEXT_OPENGL
                    || value == libretro::retro_hw_context_type_RETRO_HW_CONTEXT_OPENGL_CORE
                    || value == libretro::retro_hw_context_type_RETRO_HW_CONTEXT_OPENGLES2
                    || value == libretro::retro_hw_context_type_RETRO_HW_CONTEXT_OPENGLES3
                    || value
                        == libretro::retro_hw_context_type_RETRO_HW_CONTEXT_OPENGLES_VERSION =>
            {
                GfxBackendKind::OpenGl
            }
            _ => GfxBackendKind::Software,
        }
    }
}

/// Immutable command sent to the native wgpu renderer.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WgpuRenderCommand {
    pub native_view: u64,
    pub context: u64,
    pub framebuffer: usize,
    pub viewport: [i32; 4],
    pub output_size: [u32; 2],
    pub texture_size: [u32; 2],
    pub source_is_hardware: bool,
    pub bottom_left_origin: bool,
    pub rotation_quarters: u32,
    pub scale_mode: u32,
    pub filter_mode: u32,
    pub vsync: bool,
    pub clear_color: [f32; 4],
}

pub type WgpuRenderCallback = unsafe extern "C" fn(
    command: *const WgpuRenderCommand,
    rgba: *const u8,
    rgba_len: usize,
    user_data: *mut c_void,
) -> bool;

pub type GetProcAddressCallback = unsafe extern "C" fn(
    symbol: *const c_char,
    user_data: *mut c_void,
) -> libretro::retro_proc_address_t;

/// Opaque handles supplied by the native host.
#[derive(Debug, Clone, Copy)]
pub struct HostRenderHandles {
    pub native_view: u64,
    pub context: u64,
    pub framebuffer: usize,
    pub render_callback: Option<WgpuRenderCallback>,
    pub get_proc_address: Option<GetProcAddressCallback>,
    pub user_data: *mut c_void,
}

unsafe impl Send for HostRenderHandles {}
unsafe impl Sync for HostRenderHandles {}

impl Default for HostRenderHandles {
    fn default() -> Self {
        Self {
            native_view: 0,
            context: 0,
            framebuffer: 0,
            render_callback: None,
            get_proc_address: None,
            user_data: std::ptr::null_mut(),
        }
    }
}

impl HostRenderHandles {
    pub fn is_valid(self) -> bool {
        self.native_view != 0 && self.render_callback.is_some()
    }
}

/// Hardware frame marker emitted by libretro hardware cores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareFrame {
    pub width: u32,
    pub height: u32,
    pub frame_number: u64,
    pub request: HardwareRenderRequest,
}
