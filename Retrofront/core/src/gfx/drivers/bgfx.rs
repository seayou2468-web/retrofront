use super::{DriverFrame, GfxDriver, PresentStatus};
use crate::gfx::config::GfxVideoConfig;
use crate::gfx::context::ContextDriver;
use crate::gfx::frame::VideoFrame;
use crate::gfx::hardware::{BgfxRenderCommand, GfxBackendKind, HostRenderHandles};
use crate::gfx::CLEAR_COLOR_RGBA;
use std::ptr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BgfxDrawCall {
    pub viewport: [i32; 4],
    pub output_size: [u32; 2],
    pub texture_size: [u32; 2],
    pub framebuffer: usize,
    pub native_view: u64,
    pub context: u64,
    pub source_is_hardware: bool,
    pub bottom_left_origin: bool,
    pub rotation_quarters: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BgfxDriver {
    handles: HostRenderHandles,
    last_draw_call: Option<BgfxDrawCall>,
    video_config: GfxVideoConfig,
    initialized: bool,
}

impl BgfxDriver {
    pub fn configure(&mut self, context: &ContextDriver) {
        self.handles = context.handles();
    }

    pub fn last_draw_call(&self) -> Option<&BgfxDrawCall> {
        self.last_draw_call.as_ref()
    }

    pub fn set_video_config(&mut self, config: GfxVideoConfig) {
        self.video_config = config;
    }

    fn build_call(
        &self,
        width: u32,
        height: u32,
        bottom_left_origin: bool,
        source_is_hardware: bool,
    ) -> Result<BgfxDrawCall, String> {
        if !self.handles.is_valid() {
            return Err("bgfx backend requires valid host handles".into());
        }
        Ok(BgfxDrawCall {
            viewport: self.video_config.viewport(width, height),
            output_size: self.video_config.output_size(width, height),
            texture_size: [width, height],
            framebuffer: self.handles.framebuffer,
            native_view: self.handles.native_view,
            context: self.handles.context,
            bottom_left_origin,
            source_is_hardware,
            rotation_quarters: self.video_config.rotation_quarters % 4,
        })
    }

    fn execute(&self, call: &BgfxDrawCall, rgba: Option<&[u8]>) -> Result<(), String> {
        let callback = self
            .handles
            .render_callback
            .ok_or_else(|| "bgfx render callback was not configured".to_string())?;

        let command = BgfxRenderCommand {
            native_view: call.native_view,
            context: call.context,
            framebuffer: call.framebuffer,
            viewport: call.viewport,
            output_size: call.output_size,
            texture_size: call.texture_size,
            source_is_hardware: call.source_is_hardware,
            bottom_left_origin: call.bottom_left_origin,
            rotation_quarters: call.rotation_quarters,
            scale_mode: self.video_config.scale_mode as u32,
            filter_mode: self.video_config.filter_mode as u32,
            vsync: self.video_config.vsync,
            clear_color: CLEAR_COLOR_RGBA,
        };

        let (ptr, len) = rgba.map_or((ptr::null(), 0), |bytes| (bytes.as_ptr(), bytes.len()));
        let rendered = unsafe { callback(&command, ptr, len, self.handles.user_data) };
        if rendered {
            Ok(())
        } else {
            Err("bgfx render callback reported failure".into())
        }
    }
}

impl GfxDriver for BgfxDriver {
    fn name(&self) -> &'static str {
        "bgfx-host"
    }
    fn init(&mut self, context: &ContextDriver, _bootstrap_frame: &VideoFrame) {
        self.configure(context);
        self.initialized = true;
    }
    fn context_reset(&mut self, context: &ContextDriver) {
        self.configure(context);
        self.last_draw_call = None;
    }
    fn present(&mut self, frame: &DriverFrame) -> Result<PresentStatus, String> {
        if !self.initialized {
            self.initialized = true;
        }
        let (width, height, frame_number, bottom_left_origin, source_is_hardware, rgba) =
            match frame {
                DriverFrame::Software(frame) => (
                    frame.width,
                    frame.height,
                    0,
                    false,
                    false,
                    Some(frame.rgba.as_slice()),
                ),
                DriverFrame::Hardware(frame) => (
                    frame.width,
                    frame.height,
                    frame.frame_number,
                    frame.request.bottom_left_origin,
                    true,
                    None,
                ),
            };
        let call = self.build_call(width, height, bottom_left_origin, source_is_hardware)?;
        self.execute(&call, rgba)?;
        self.last_draw_call = Some(call);
        Ok(PresentStatus {
            backend: GfxBackendKind::Bgfx,
            frame_number,
            rendered: true,
            details: format!("bgfx rendered {width}x{height} through host callback"),
        })
    }
    fn destroy(&mut self) {
        self.initialized = false;
        self.last_draw_call = None;
    }
}
