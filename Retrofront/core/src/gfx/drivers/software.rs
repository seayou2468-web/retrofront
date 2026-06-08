use super::{DriverFrame, GfxDriver, PresentStatus};
use crate::gfx::config::GfxVideoConfig;
use crate::gfx::context::ContextDriver;
use crate::gfx::frame::VideoFrame;
use crate::gfx::hardware::GfxBackendKind;

#[derive(Debug, Clone, Default)]
pub struct SoftwareDriver {
    last_frame: Option<VideoFrame>,
    video_config: GfxVideoConfig,
}

impl SoftwareDriver {
    pub fn last_frame(&self) -> Option<&VideoFrame> {
        self.last_frame.as_ref()
    }

    pub fn set_video_config(&mut self, config: GfxVideoConfig) {
        self.video_config = config;
    }
}

impl GfxDriver for SoftwareDriver {
    fn name(&self) -> &'static str {
        "software"
    }
    fn init(&mut self, _context: &ContextDriver, bootstrap_frame: &VideoFrame) {
        if !bootstrap_frame.rgba.is_empty() {
            self.last_frame = Some(bootstrap_frame.clone());
        }
    }
    fn context_reset(&mut self, _context: &ContextDriver) {}
    fn present(&mut self, frame: &DriverFrame) -> Result<PresentStatus, String> {
        match frame {
            DriverFrame::Software(frame) => {
                self.last_frame = Some(frame.clone());
                Ok(PresentStatus {
                    backend: GfxBackendKind::Software,
                    frame_number: 0,
                    rendered: true,
                    details: format!("{}x{} RGBA frame cached", frame.width, frame.height),
                })
            }
            DriverFrame::Hardware(_) => {
                Err("software driver cannot present a hardware framebuffer".into())
            }
        }
    }
    fn destroy(&mut self) {
        self.last_frame = None;
    }
}
