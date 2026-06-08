pub mod config;
pub mod context;
pub mod drivers;
pub mod frame;
pub mod hardware;

use crate::libretro;
pub use config::{GfxFilterMode, GfxScaleMode, GfxVideoConfig};
pub use context::ContextDriver;
pub use drivers::bgfx::{BgfxDrawCall, BgfxDriver};
use drivers::software::SoftwareDriver;
pub use drivers::{DriverFrame, GfxDriver, PresentStatus};
pub use frame::{PixelFormat, VideoFrame};
pub use hardware::{
    GfxBackendKind, HardwareFrame, HardwareRenderRequest, HostRenderHandles,
    BgfxRenderCommand,
};

pub const CLEAR_COLOR_RGBA: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

#[derive(Debug, Clone, Default)]
pub struct GfxStatus {
    pub last_present: Option<PresentStatus>,
    pub frame_counter: u64,
    pub hardware_ready: bool,
}

pub struct GfxRuntime {
    backend: GfxBackendKind,
    pub software: SoftwareDriver,
    pub bgfx: BgfxDriver,
    pub context: ContextDriver,
    pixel_format: PixelFormat,
    video_config: GfxVideoConfig,
    status: GfxStatus,
}

impl GfxRuntime {
    pub fn new() -> Self {
        Self {
            backend: GfxBackendKind::Software,
            software: SoftwareDriver::default(),
            bgfx: BgfxDriver::default(),
            context: ContextDriver::new(),
            pixel_format: PixelFormat::ZeroRgb1555,
            video_config: GfxVideoConfig::default(),
            status: GfxStatus::default(),
        }
    }

    pub fn set_backend(&mut self, backend: GfxBackendKind) {
        self.backend = backend;
    }

    pub fn set_pixel_format(&mut self, format: PixelFormat) {
        self.pixel_format = format;
    }

    pub fn update_system_av_info(&mut self, av_info: &crate::libretro::retro_system_av_info) {
        let config = GfxVideoConfig::from_libretro_geometry(&av_info.geometry);
        self.set_video_config(config);
    }


    pub fn video_config(&self) -> GfxVideoConfig {
        self.video_config
    }
    pub fn set_video_config(&mut self, config: GfxVideoConfig) {
        self.video_config = config;
        self.software.set_video_config(config);
        self.bgfx.set_video_config(config);
    }

    pub fn set_host_handles(&mut self, handles: HostRenderHandles) {
        self.context.set_handles(handles);
        self.bgfx.configure(&self.context);
        self.status.hardware_ready = self.context.hardware_ready();
    }

    pub fn set_hardware_render_request(&mut self, request: HardwareRenderRequest) {
        self.context.set_hardware_request(request);
        self.backend = request.preferred_backend();
        self.status.hardware_ready = self.context.hardware_ready();
    }

    pub fn driver_status(&self) -> &GfxStatus {
        &self.status
    }

    pub fn ingest_software_frame(
        &mut self,
        data: *const u8,
        width: u32,
        height: u32,
        pitch: usize,
    ) -> Result<VideoFrame, String> {
        let frame = frame::convert_frame_to_rgba(data.cast(), width, height, pitch, self.pixel_format)?;
        let status = self.driver_mut().present(&DriverFrame::Software(frame.clone()))?;
        self.status.last_present = Some(status);
        self.status.frame_counter = self.status.frame_counter.wrapping_add(1);
        Ok(frame)
    }

    pub fn ingest_hardware_frame(&mut self, width: u32, height: u32) -> Result<(), String> {
        let request = self
            .context
            .hardware_request()
            .ok_or_else(|| "hardware frame received but no request active".to_string())?;
        let frame = HardwareFrame {
            width,
            height,
            frame_number: self.status.frame_counter,
            request,
        };
        let status = self.driver_mut().present(&DriverFrame::Hardware(frame))?;
        self.status.last_present = Some(status);
        self.status.frame_counter = self.status.frame_counter.wrapping_add(1);
        Ok(())
    }

    pub fn patch_hw_render_callback(&mut self, raw: &mut libretro::retro_hw_render_callback) {
        self.context.capture_callbacks(raw);
        self.context.patch_hw_render_callback(raw);
        if self.context.hardware_ready() {
            self.context.notify_reset();
            self.status.hardware_ready = true;
        }
    }

    pub fn bgfx_draw_call(&self) -> Option<&BgfxDrawCall> {
        self.bgfx.last_draw_call()
    }

    pub fn context_handles(&self) -> HostRenderHandles {
        self.context.handles()
    }

    pub fn last_frame(&self) -> &VideoFrame {
        static EMPTY_FRAME: VideoFrame = VideoFrame {
            width: 0,
            height: 0,
            pitch: 0,
            source_format: PixelFormat::ZeroRgb1555,
            rgba: Vec::new(),
        };
        self.software.last_frame().unwrap_or(&EMPTY_FRAME)
    }

    pub fn frame_counter(&self) -> u64 {
        self.status.frame_counter
    }

    fn driver(&self) -> &dyn GfxDriver {
        match self.backend {
            GfxBackendKind::Software => &self.software,
            GfxBackendKind::Bgfx => &self.bgfx,
        }
    }

    fn driver_mut(&mut self) -> &mut dyn GfxDriver {
        match self.backend {
            GfxBackendKind::Software => &mut self.software,
            GfxBackendKind::Bgfx => &mut self.bgfx,
        }
    }
}

impl GfxBackendKind {
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Software),
            1 => Some(Self::Bgfx),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn test_bgfx_render(
        command: *const BgfxRenderCommand,
        _rgba: *const u8,
        _rgba_len: usize,
        _user_data: *mut std::os::raw::c_void,
    ) -> bool {
        !command.is_null()
    }

    unsafe extern "C" fn counting_bgfx_render(
        command: *const BgfxRenderCommand,
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
            gfx.driver_status().last_present.as_ref().unwrap().backend,
            GfxBackendKind::Software
        );
    }

    #[test]
    fn backend_codes_are_stable() {
        assert_eq!(GfxBackendKind::from_code(0), Some(GfxBackendKind::Software));
        assert_eq!(GfxBackendKind::from_code(1), Some(GfxBackendKind::Bgfx));
        assert_eq!(GfxBackendKind::from_code(99), None);
    }

    #[test]
    fn bgfx_hardware_frame_builds_draw_call() {
        let mut gfx = GfxRuntime::new();
        gfx.set_hardware_render_request(HardwareRenderRequest::opengles(3, 0));
        gfx.set_host_handles(HostRenderHandles {
            native_view: 7,
            context: 9,
            framebuffer: 13,
            render_callback: Some(test_bgfx_render),
            ..HostRenderHandles::default()
        });
        gfx.ingest_hardware_frame(320, 240).expect("hardware frame");
        let call = gfx.bgfx_draw_call().expect("draw call");
        assert_eq!(call.viewport, [0, 0, 320, 240]);
        assert_eq!(call.framebuffer, 13);
    }

    #[test]
    fn bgfx_software_frame_invokes_host_renderer_with_rgba() {
        let mut gfx = GfxRuntime::new();
        let mut uploaded_len = 0usize;
        gfx.set_backend(GfxBackendKind::Bgfx);
        gfx.set_host_handles(HostRenderHandles {
            native_view: 1,
            context: 2,
            framebuffer: 3,
            render_callback: Some(counting_bgfx_render),
            user_data: (&mut uploaded_len as *mut usize).cast(),
            ..HostRenderHandles::default()
        });
        gfx.set_pixel_format(PixelFormat::Xrgb8888);
        let pixel = 0x0012_3456u32.to_ne_bytes();
        gfx.ingest_software_frame(pixel.as_ptr().cast(), 1, 1, 4)
            .expect("bgfx software upload");
        assert_eq!(uploaded_len, 4);
        assert_eq!(gfx.bgfx_draw_call().expect("draw call").framebuffer, 3);
    }

    #[test]
    fn video_config_controls_bgfx_viewport_and_sampling() {
        let mut gfx = GfxRuntime::new();
        gfx.set_backend(GfxBackendKind::Bgfx);
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
            context: 2,
            framebuffer: 3,
            render_callback: Some(test_bgfx_render),
            ..HostRenderHandles::default()
        });
        gfx.set_pixel_format(PixelFormat::Xrgb8888);
        let pixels = vec![0u8; 320 * 240 * 4];
        gfx.ingest_software_frame(pixels.as_ptr().cast(), 320, 240, 320 * 4)
            .expect("bgfx configured render");
        let call = gfx.bgfx_draw_call().expect("draw call");
        assert_eq!(call.output_size, [1280, 720]);
        assert_eq!(call.viewport, [160, 0, 960, 720]);
        assert_eq!(call.rotation_quarters, 1);
    }
}
