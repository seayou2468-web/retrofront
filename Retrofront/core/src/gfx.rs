//! Portable libretro video path for Retrofront.
//!
//! This module is the Rust equivalent of the first layer of RetroArch's `gfx`:
//! it receives video frames from `retro_video_refresh_t`, normalizes libretro
//! pixel formats, owns the latest frame buffer, and exposes backend selection
//! metadata to platform UIs. The actual native swapchain/view is still supplied
//! by iOS or Linux, but the core frame ingestion/conversion path is shared Rust.

use crate::libretro;
use std::os::raw::c_void;

const RETRO_HW_FRAME_BUFFER_VALID: *const c_void = usize::MAX as *const c_void;

/// Rendering backend requested by the host application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GfxBackendKind {
    /// CPU-backed path. Always available and used as the compatibility fallback.
    Software = 0,
    /// OpenGL/OpenGL ES texture upload path. iOS uses SDK OpenGL ES contexts.
    OpenGl = 1,
    /// Vulkan path. iOS hosts this through MoltenVK-provided Vulkan surfaces.
    Vulkan = 2,
}

impl GfxBackendKind {
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Software),
            1 => Some(Self::OpenGl),
            2 => Some(Self::Vulkan),
            _ => None,
        }
    }
}

/// Libretro software pixel formats accepted by `RETRO_ENVIRONMENT_SET_PIXEL_FORMAT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    ZeroRgb1555,
    Xrgb8888,
    Rgb565,
}

impl Default for PixelFormat {
    fn default() -> Self {
        Self::ZeroRgb1555
    }
}

impl PixelFormat {
    pub fn from_libretro(value: u32) -> Option<Self> {
        match value {
            value if value == libretro::retro_pixel_format_RETRO_PIXEL_FORMAT_0RGB1555 => {
                Some(Self::ZeroRgb1555)
            }
            value if value == libretro::retro_pixel_format_RETRO_PIXEL_FORMAT_XRGB8888 => {
                Some(Self::Xrgb8888)
            }
            value if value == libretro::retro_pixel_format_RETRO_PIXEL_FORMAT_RGB565 => {
                Some(Self::Rgb565)
            }
            _ => None,
        }
    }

    pub fn bytes_per_pixel(self) -> usize {
        match self {
            Self::ZeroRgb1555 | Self::Rgb565 => 2,
            Self::Xrgb8888 => 4,
        }
    }

    pub fn code(self) -> u32 {
        match self {
            Self::ZeroRgb1555 => 0,
            Self::Xrgb8888 => 1,
            Self::Rgb565 => 2,
        }
    }
}

/// Latest frame normalized for direct upload/display by platform code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub pitch: usize,
    pub source_format: PixelFormat,
    /// Tight RGBA8888, top-left origin, one row after another.
    pub rgba: Vec<u8>,
}

impl Default for VideoFrame {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            pitch: 0,
            source_format: PixelFormat::default(),
            rgba: Vec::new(),
        }
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

/// Portable renderer state owned by the Rust runtime.
#[derive(Debug, Clone)]
pub struct GfxRuntime {
    backend: GfxBackendKind,
    pixel_format: PixelFormat,
    last_frame: VideoFrame,
    hw_render: Option<HardwareRenderRequest>,
    frame_counter: u64,
}

impl Default for GfxRuntime {
    fn default() -> Self {
        Self {
            backend: GfxBackendKind::Software,
            pixel_format: PixelFormat::default(),
            last_frame: VideoFrame::default(),
            hw_render: None,
            frame_counter: 0,
        }
    }
}

impl GfxRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn backend(&self) -> GfxBackendKind {
        self.backend
    }

    pub fn set_backend(&mut self, backend: GfxBackendKind) {
        self.backend = backend;
    }

    pub fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    pub fn set_pixel_format(&mut self, format: PixelFormat) {
        self.pixel_format = format;
    }

    pub fn last_frame(&self) -> &VideoFrame {
        &self.last_frame
    }

    pub fn frame_counter(&self) -> u64 {
        self.frame_counter
    }

    pub fn hardware_render_request(&self) -> Option<HardwareRenderRequest> {
        self.hw_render
    }

    pub fn set_hardware_render_request(&mut self, request: HardwareRenderRequest) {
        self.hw_render = Some(request);
    }

    pub fn clear_hardware_render_request(&mut self) {
        self.hw_render = None;
    }

    /// Copies the core-provided software frame and normalizes it to RGBA8888.
    pub fn ingest_software_frame(
        &mut self,
        data: *const c_void,
        width: u32,
        height: u32,
        pitch: usize,
    ) -> Result<&VideoFrame, String> {
        if width == 0 || height == 0 {
            self.last_frame = VideoFrame {
                width,
                height,
                pitch,
                source_format: self.pixel_format,
                rgba: Vec::new(),
            };
            self.frame_counter = self.frame_counter.saturating_add(1);
            return Ok(&self.last_frame);
        }
        if data.is_null() || data == RETRO_HW_FRAME_BUFFER_VALID {
            return Err(
                "hardware frames require a platform GL/Vulkan surface before display".into(),
            );
        }
        let min_pitch = width as usize * self.pixel_format.bytes_per_pixel();
        if pitch < min_pitch {
            return Err(format!(
                "frame pitch {} is smaller than required {} for {}x{}",
                pitch, min_pitch, width, height
            ));
        }
        let len = pitch
            .checked_mul(height as usize)
            .ok_or_else(|| "frame size overflow".to_string())?;
        let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len) };
        let mut rgba = vec![0; width as usize * height as usize * 4];
        convert_to_rgba(
            self.pixel_format,
            bytes,
            width as usize,
            height as usize,
            pitch,
            &mut rgba,
        );
        self.last_frame = VideoFrame {
            width,
            height,
            pitch,
            source_format: self.pixel_format,
            rgba,
        };
        self.frame_counter = self.frame_counter.saturating_add(1);
        Ok(&self.last_frame)
    }
}

fn convert_to_rgba(
    format: PixelFormat,
    src: &[u8],
    width: usize,
    height: usize,
    pitch: usize,
    dst: &mut [u8],
) {
    for y in 0..height {
        let row = &src[y * pitch..];
        for x in 0..width {
            let out = (y * width + x) * 4;
            match format {
                PixelFormat::Xrgb8888 => {
                    let pixel = u32::from_ne_bytes([
                        row[x * 4],
                        row[x * 4 + 1],
                        row[x * 4 + 2],
                        row[x * 4 + 3],
                    ]);
                    dst[out] = ((pixel >> 16) & 0xff) as u8;
                    dst[out + 1] = ((pixel >> 8) & 0xff) as u8;
                    dst[out + 2] = (pixel & 0xff) as u8;
                    dst[out + 3] = 0xff;
                }
                PixelFormat::Rgb565 => {
                    let pixel = u16::from_ne_bytes([row[x * 2], row[x * 2 + 1]]);
                    dst[out] = expand_5(((pixel >> 11) & 0x1f) as u8);
                    dst[out + 1] = expand_6(((pixel >> 5) & 0x3f) as u8);
                    dst[out + 2] = expand_5((pixel & 0x1f) as u8);
                    dst[out + 3] = 0xff;
                }
                PixelFormat::ZeroRgb1555 => {
                    let pixel = u16::from_ne_bytes([row[x * 2], row[x * 2 + 1]]);
                    dst[out] = expand_5(((pixel >> 10) & 0x1f) as u8);
                    dst[out + 1] = expand_5(((pixel >> 5) & 0x1f) as u8);
                    dst[out + 2] = expand_5((pixel & 0x1f) as u8);
                    dst[out + 3] = 0xff;
                }
            }
        }
    }
}

fn expand_5(value: u8) -> u8 {
    (value << 3) | (value >> 2)
}

fn expand_6(value: u8) -> u8 {
    (value << 2) | (value >> 4)
}

pub unsafe fn hw_render_request_from_raw(
    raw: *mut libretro::retro_hw_render_callback,
) -> Option<HardwareRenderRequest> {
    let raw = unsafe { raw.as_mut()? };
    raw.get_current_framebuffer = Some(get_current_framebuffer);
    raw.get_proc_address = Some(get_proc_address);
    Some(HardwareRenderRequest {
        context_type: raw.context_type,
        version_major: raw.version_major,
        version_minor: raw.version_minor,
        depth: raw.depth,
        stencil: raw.stencil,
        bottom_left_origin: raw.bottom_left_origin,
        cache_context: raw.cache_context,
        debug_context: raw.debug_context,
    })
}

unsafe extern "C" fn get_current_framebuffer() -> usize {
    usize::MAX
}

unsafe extern "C" fn get_proc_address(_sym: *const i8) -> libretro::retro_proc_address_t {
    None
}

/// Clear color used by both OpenGL and Vulkan upload pipelines before drawing.
pub const CLEAR_COLOR_RGBA: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// Minimal GLSL vertex shader shared by OpenGL/OpenGL ES texture upload hosts.
pub const OPENGL_VERTEX_SHADER: &str = r#"#version 300 es
precision mediump float;
layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texcoord;
out vec2 v_texcoord;
void main() {
    v_texcoord = a_texcoord;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;

/// Minimal GLSL fragment shader that displays the Rust-normalized RGBA frame.
pub const OPENGL_FRAGMENT_SHADER: &str = r#"#version 300 es
precision mediump float;
in vec2 v_texcoord;
uniform sampler2D u_frame;
out vec4 color;
void main() {
    color = texture(u_frame, v_texcoord);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_rgb565_to_rgba() {
        let mut gfx = GfxRuntime::new();
        gfx.set_pixel_format(PixelFormat::Rgb565);
        let pixels = [0x00u8, 0xf8, 0xe0, 0x07];
        let frame = gfx
            .ingest_software_frame(pixels.as_ptr().cast(), 2, 1, 4)
            .expect("valid frame");
        assert_eq!(frame.rgba, vec![255, 0, 0, 255, 0, 255, 0, 255]);
    }

    #[test]
    fn converts_xrgb8888_to_rgba() {
        let mut gfx = GfxRuntime::new();
        gfx.set_pixel_format(PixelFormat::Xrgb8888);
        let pixel = 0x0012_3456u32.to_ne_bytes();
        let frame = gfx
            .ingest_software_frame(pixel.as_ptr().cast(), 1, 1, 4)
            .expect("valid frame");
        assert_eq!(frame.rgba, vec![0x12, 0x34, 0x56, 0xff]);
    }

    #[test]
    fn rejects_short_pitch() {
        let mut gfx = GfxRuntime::new();
        gfx.set_pixel_format(PixelFormat::Xrgb8888);
        let pixels = [0u8; 8];
        assert!(gfx
            .ingest_software_frame(pixels.as_ptr().cast(), 2, 1, 4)
            .is_err());
    }

    #[test]
    fn backend_codes_are_stable() {
        assert_eq!(GfxBackendKind::from_code(0), Some(GfxBackendKind::Software));
        assert_eq!(GfxBackendKind::from_code(1), Some(GfxBackendKind::OpenGl));
        assert_eq!(GfxBackendKind::from_code(2), Some(GfxBackendKind::Vulkan));
        assert_eq!(GfxBackendKind::from_code(99), None);
    }
}
