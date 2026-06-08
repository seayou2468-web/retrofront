//! RetroArch-style graphics layer for Retrofront.
//!
//! The module is intentionally split like RetroArch's `gfx/` tree: a context
//! driver captures libretro hardware-render requests, backend drivers implement a
//! common `GfxDriver` trait, and the runtime routes software frames plus GL/Vulkan
//! hardware frames through one state machine. Platform code still owns native
//! windows/layers, but the Rust side now provides complete driver selection,
//! frame conversion, command generation, callbacks, shader sources, and
//! MoltenVK/iOS SDK metadata rather than a single ad-hoc upload helper.

pub mod config;
pub mod context;
pub mod drivers;
pub mod frame;
pub mod hardware;

use crate::libretro;
pub use config::{GfxFilterMode, GfxScaleMode, GfxVideoConfig};
use context::{ContextDriver, ContextEvent};
pub use drivers::opengl::{GlDrawCall, OpenGlDriver, OPENGL_FRAGMENT_SHADER, OPENGL_VERTEX_SHADER};
pub use drivers::software::SoftwareDriver;
pub use drivers::vulkan::{VulkanDriver, VulkanPresentPlan};
use drivers::{DriverFrame, GfxDriver, PresentStatus};
pub use frame::{convert_frame_to_rgba, PixelFormat, VideoFrame, RETRO_HW_FRAME_BUFFER_VALID};
pub use hardware::{
    GfxBackendKind, HardwareFrame, HardwareRenderRequest, HostRenderHandles, OpenGlRenderCommand,
    VulkanRenderCommand,
};
use std::os::raw::c_void;

/// Clear color used by all drivers before drawing a frame.
pub const CLEAR_COLOR_RGBA: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// Snapshot of the selected backend and its last present operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GfxDriverStatus {
    pub backend: GfxBackendKind,
    pub name: &'static str,
    pub frame_counter: u64,
    pub hardware_ready: bool,
    pub last_present: Option<PresentStatus>,
}

/// Portable renderer state owned by the Rust runtime.
#[derive(Debug, Clone)]
pub struct GfxRuntime {
    backend: GfxBackendKind,
    pixel_format: PixelFormat,
    last_frame: VideoFrame,
    context: ContextDriver,
    video_config: GfxVideoConfig,
    software: SoftwareDriver,
    opengl: OpenGlDriver,
    vulkan: VulkanDriver,
    frame_counter: u64,
    last_present: Option<PresentStatus>,
}

impl Default for GfxRuntime {
    fn default() -> Self {
        Self {
            backend: GfxBackendKind::Software,
            pixel_format: PixelFormat::default(),
            last_frame: VideoFrame::default(),
            context: ContextDriver::default(),
            video_config: GfxVideoConfig::default(),
            software: SoftwareDriver::default(),
            opengl: OpenGlDriver::default(),
            vulkan: VulkanDriver::default(),
            frame_counter: 0,
            last_present: None,
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
        if self.backend == backend {
            return;
        }
        self.driver_mut().destroy();
        self.backend = backend;
        let context = self.context.clone();
        let frame = self.last_frame.clone();
        self.driver_mut().init(&context, &frame);
    }

    pub fn driver_status(&self) -> GfxDriverStatus {
        let driver = self.driver();
        GfxDriverStatus {
            backend: self.backend,
            name: driver.name(),
            frame_counter: self.frame_counter,
            hardware_ready: self.context.hardware_ready(),
            last_present: self.last_present.clone(),
        }
    }

    pub fn video_config(&self) -> GfxVideoConfig {
        self.video_config
    }

    pub fn set_video_config(&mut self, config: GfxVideoConfig) {
        self.video_config = config;
        self.software.set_video_config(config);
        self.opengl.set_video_config(config);
        self.vulkan.set_video_config(config);
    }

    pub fn update_geometry(&mut self, geometry: &libretro::retro_game_geometry) {
        let output = self
            .video_config
            .output_size(geometry.base_width, geometry.base_height);
        self.set_video_config(
            GfxVideoConfig::from_libretro_geometry(geometry).with_output_size(output[0], output[1]),
        );
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
        self.context.request()
    }

    pub fn set_hardware_render_request(&mut self, request: HardwareRenderRequest) {
        self.context.set_request(request);
        self.backend = request.preferred_backend();
        let context = self.context.clone();
        let frame = self.last_frame.clone();
        self.driver_mut().init(&context, &frame);
    }

    pub fn clear_hardware_render_request(&mut self) {
        self.driver_mut().destroy();
        self.context.clear_request();
    }

    pub fn context_handles(&self) -> HostRenderHandles {
        self.context.handles()
    }

    pub fn set_host_handles(&mut self, handles: HostRenderHandles) {
        self.context.set_host_handles(handles);
        let context = self.context.clone();
        self.opengl.configure(&context);
        self.vulkan.configure(&context);
        if self.context.hardware_ready() {
            self.context.notify_reset();
        }
    }

    pub fn context_event(&mut self, event: ContextEvent) {
        self.context.apply_event(event);
        let context = self.context.clone();
        self.driver_mut().context_reset(&context);
    }

    /// Copies a core-provided software frame, normalizes it to RGBA8888, and
    /// immediately presents it through the active driver command path.
    pub fn ingest_software_frame(
        &mut self,
        data: *const c_void,
        width: u32,
        height: u32,
        pitch: usize,
    ) -> Result<&VideoFrame, String> {
        let frame = convert_frame_to_rgba(data, width, height, pitch, self.pixel_format)?;
        self.last_frame = frame;
        self.frame_counter = self.frame_counter.saturating_add(1);
        let driver_frame = DriverFrame::Software(self.last_frame.clone());
        self.last_present = Some(self.driver_mut().present(&driver_frame)?);
        Ok(&self.last_frame)
    }

    /// Records a libretro hardware frame (`RETRO_HW_FRAME_BUFFER_VALID`) and
    /// drives the active GL/Vulkan backend with the currently registered host
    /// handles.
    pub fn ingest_hardware_frame(&mut self, width: u32, height: u32) -> Result<(), String> {
        let request = self
            .context
            .request()
            .ok_or_else(|| "hardware frame received before SET_HW_RENDER".to_string())?;
        let frame = HardwareFrame {
            width,
            height,
            frame_number: self.frame_counter.saturating_add(1),
            request,
        };
        self.frame_counter = frame.frame_number;
        self.last_present = Some(self.driver_mut().present(&DriverFrame::Hardware(frame))?);
        Ok(())
    }

    /// Fills a libretro callback structure with the frontend callbacks that the
    /// active context driver owns. Mirrors RetroArch's SET_HW_RENDER handling.
    pub fn patch_hw_render_callback(&mut self, raw: &mut libretro::retro_hw_render_callback) {
        self.context.capture_callbacks(raw);
        self.context.patch_hw_render_callback(raw);
        if self.context.hardware_ready() {
            self.context.notify_reset();
        }
    }

    pub fn opengl_draw_call(&self) -> Option<&GlDrawCall> {
        self.opengl.last_draw_call()
    }

    pub fn vulkan_present_plan(&self) -> Option<&VulkanPresentPlan> {
        self.vulkan.last_present_plan()
    }

    fn driver(&self) -> &dyn GfxDriver {
        match self.backend {
            GfxBackendKind::Software => &self.software,
            GfxBackendKind::OpenGl => &self.opengl,
            GfxBackendKind::Vulkan => &self.vulkan,
        }
    }

    fn driver_mut(&mut self) -> &mut dyn GfxDriver {
        match self.backend {
            GfxBackendKind::Software => &mut self.software,
            GfxBackendKind::OpenGl => &mut self.opengl,
            GfxBackendKind::Vulkan => &mut self.vulkan,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn test_gl_render(
        command: *const OpenGlRenderCommand,
        _rgba: *const u8,
        _rgba_len: usize,
        _user_data: *mut std::os::raw::c_void,
    ) -> bool {
        !command.is_null()
    }

    unsafe extern "C" fn test_vulkan_render(
        command: *const VulkanRenderCommand,
        _rgba: *const u8,
        _rgba_len: usize,
        _user_data: *mut std::os::raw::c_void,
    ) -> bool {
        !command.is_null()
    }

    unsafe extern "C" fn counting_gl_render(
        command: *const OpenGlRenderCommand,
        _rgba: *const u8,
        rgba_len: usize,
        user_data: *mut std::os::raw::c_void,
    ) -> bool {
        if command.is_null() || user_data.is_null() {
            return false;
        }
        unsafe {
            assert_eq!((*command).texture_size, [1, 1]);
            *(user_data.cast::<usize>()) = rgba_len;
        }
        true
    }

    unsafe extern "C" fn counting_vulkan_render(
        command: *const VulkanRenderCommand,
        _rgba: *const u8,
        rgba_len: usize,
        user_data: *mut std::os::raw::c_void,
    ) -> bool {
        if command.is_null() || user_data.is_null() {
            return false;
        }
        unsafe {
            assert_eq!((*command).extent, [1, 1]);
            *(user_data.cast::<usize>()) = rgba_len;
        }
        true
    }

    #[test]
    fn converts_rgb565_to_rgba() {
        let mut gfx = GfxRuntime::new();
        gfx.set_pixel_format(PixelFormat::Rgb565);
        let pixels = [0x00u8, 0xf8, 0xe0, 0x07];
        let frame = gfx
            .ingest_software_frame(pixels.as_ptr().cast(), 2, 1, 4)
            .expect("valid frame");
        assert_eq!(frame.rgba, vec![255, 0, 0, 255, 0, 255, 0, 255]);
        assert_eq!(
            gfx.driver_status().last_present.unwrap().backend,
            GfxBackendKind::Software
        );
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

    #[test]
    fn opengl_hardware_frame_builds_draw_call() {
        let mut gfx = GfxRuntime::new();
        gfx.set_hardware_render_request(HardwareRenderRequest::opengles(3, 0));
        gfx.set_host_handles(HostRenderHandles {
            native_view: 7,
            gl_context: 9,
            gl_framebuffer: 13,
            opengl_render: Some(test_gl_render),
            vulkan_instance: 0,
            vulkan_device: 0,
            vulkan_queue: 0,
            vulkan_command_buffer: 0,
            vulkan_image: 0,
            ..HostRenderHandles::default()
        });
        gfx.ingest_hardware_frame(320, 240).expect("hardware frame");
        let call = gfx.opengl_draw_call().expect("draw call");
        assert_eq!(call.viewport, [0, 0, 320, 240]);
        assert_eq!(call.framebuffer, 13);
        assert!(call.uses_ios_sdk_context);
    }

    #[test]
    fn opengl_software_frame_invokes_host_renderer_with_rgba() {
        let mut gfx = GfxRuntime::new();
        let mut uploaded_len = 0usize;
        gfx.set_backend(GfxBackendKind::OpenGl);
        gfx.set_host_handles(HostRenderHandles {
            native_view: 1,
            gl_context: 2,
            gl_framebuffer: 3,
            opengl_render: Some(counting_gl_render),
            user_data: (&mut uploaded_len as *mut usize).cast(),
            ..HostRenderHandles::default()
        });
        gfx.set_pixel_format(PixelFormat::Xrgb8888);
        let pixel = 0x0001_0203u32.to_ne_bytes();
        gfx.ingest_software_frame(pixel.as_ptr().cast(), 1, 1, 4)
            .expect("OpenGL software upload");
        assert_eq!(uploaded_len, 4);
        assert_eq!(gfx.opengl_draw_call().expect("draw call").framebuffer, 3);
    }

    #[test]
    fn vulkan_software_frame_invokes_host_renderer_with_rgba() {
        let mut gfx = GfxRuntime::new();
        let mut uploaded_len = 0usize;
        gfx.set_backend(GfxBackendKind::Vulkan);
        gfx.set_host_handles(HostRenderHandles {
            native_view: 1,
            vulkan_instance: 2,
            vulkan_device: 3,
            vulkan_queue: 4,
            vulkan_command_buffer: 5,
            vulkan_image: 6,
            vulkan_render: Some(counting_vulkan_render),
            user_data: (&mut uploaded_len as *mut usize).cast(),
            ..HostRenderHandles::default()
        });
        gfx.set_pixel_format(PixelFormat::Xrgb8888);
        let pixel = 0x0001_0203u32.to_ne_bytes();
        gfx.ingest_software_frame(pixel.as_ptr().cast(), 1, 1, 4)
            .expect("Vulkan software upload");
        assert_eq!(uploaded_len, 4);
        assert_eq!(gfx.vulkan_present_plan().expect("plan").image, 6);
    }

    #[test]
    fn video_config_controls_opengl_viewport_and_sampling() {
        let mut gfx = GfxRuntime::new();
        gfx.set_backend(GfxBackendKind::OpenGl);
        gfx.set_video_config(GfxVideoConfig {
            output_width: 1280,
            output_height: 720,
            aspect_ratio: 4.0 / 3.0,
            scale_mode: GfxScaleMode::KeepAspect,
            filter_mode: GfxFilterMode::Linear,
            rotation_quarters: 1,
            ..GfxVideoConfig::default()
        });
        gfx.set_host_handles(HostRenderHandles {
            native_view: 1,
            gl_context: 2,
            gl_framebuffer: 3,
            opengl_render: Some(test_gl_render),
            ..HostRenderHandles::default()
        });
        gfx.set_pixel_format(PixelFormat::Xrgb8888);
        let pixels = vec![0u8; 320 * 240 * 4];
        gfx.ingest_software_frame(pixels.as_ptr().cast(), 320, 240, 320 * 4)
            .expect("OpenGL configured render");
        let call = gfx.opengl_draw_call().expect("draw call");
        assert_eq!(call.output_size, [1280, 720]);
        assert_eq!(call.viewport, [160, 0, 960, 720]);
        assert_eq!(call.rotation_quarters, 1);
    }

    #[test]
    fn video_config_controls_vulkan_integer_viewport() {
        let mut gfx = GfxRuntime::new();
        gfx.set_backend(GfxBackendKind::Vulkan);
        gfx.set_video_config(GfxVideoConfig {
            output_width: 1000,
            output_height: 700,
            scale_mode: GfxScaleMode::Integer,
            ..GfxVideoConfig::default()
        });
        gfx.set_host_handles(HostRenderHandles {
            native_view: 1,
            vulkan_instance: 2,
            vulkan_device: 3,
            vulkan_queue: 4,
            vulkan_command_buffer: 5,
            vulkan_image: 6,
            vulkan_render: Some(test_vulkan_render),
            ..HostRenderHandles::default()
        });
        gfx.set_pixel_format(PixelFormat::Xrgb8888);
        let pixels = vec![0u8; 320 * 240 * 4];
        gfx.ingest_software_frame(pixels.as_ptr().cast(), 320, 240, 320 * 4)
            .expect("Vulkan configured render");
        let plan = gfx.vulkan_present_plan().expect("plan");
        assert_eq!(plan.extent, [1000, 700]);
        assert_eq!(plan.viewport, [180, 110, 640, 480]);
    }

    #[test]
    fn vulkan_hardware_frame_builds_moltenvk_plan() {
        let mut gfx = GfxRuntime::new();
        gfx.set_hardware_render_request(HardwareRenderRequest::vulkan());
        gfx.set_host_handles(HostRenderHandles {
            native_view: 11,
            gl_context: 0,
            gl_framebuffer: 0,
            vulkan_render: Some(test_vulkan_render),
            vulkan_instance: 1,
            vulkan_device: 2,
            vulkan_queue: 3,
            vulkan_command_buffer: 4,
            vulkan_image: 5,
            ..HostRenderHandles::default()
        });
        gfx.ingest_hardware_frame(640, 480).expect("hardware frame");
        let plan = gfx.vulkan_present_plan().expect("present plan");
        assert!(plan.uses_moltenvk);
        assert_eq!(plan.extent, [640, 480]);
    }
}
