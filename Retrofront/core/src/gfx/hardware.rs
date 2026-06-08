use crate::libretro;

/// Rendering backend requested by the host application or libretro core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GfxBackendKind {
    /// CPU-backed path. Always available and used as compatibility fallback.
    Software = 0,
    /// OpenGL/OpenGL ES texture upload path. iOS uses SDK OpenGL ES contexts.
    OpenGl = 1,
    /// Vulkan path. Apple targets are hosted through MoltenVK surfaces.
    Vulkan = 2,
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

/// Opaque handles supplied by the native host. They map to GLK/EAGL/CAEAGL or
/// Vulkan/MoltenVK objects depending on the backend and are deliberately stored
/// as integers so the Rust static library remains portable and ABI-stable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HostRenderHandles {
    pub native_view: u64,
    pub gl_context: u64,
    pub vulkan_instance: u64,
    pub vulkan_device: u64,
    pub vulkan_queue: u64,
    pub vulkan_command_buffer: u64,
    pub vulkan_image: u64,
}

impl HostRenderHandles {
    pub fn has_opengl(self) -> bool {
        self.native_view != 0 && self.gl_context != 0
    }

    pub fn has_vulkan(self) -> bool {
        self.native_view != 0
            && self.vulkan_instance != 0
            && self.vulkan_device != 0
            && self.vulkan_queue != 0
            && self.vulkan_command_buffer != 0
            && self.vulkan_image != 0
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
