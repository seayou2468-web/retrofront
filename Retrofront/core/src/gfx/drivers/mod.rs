pub mod software;
pub mod wgpu;

use super::context::ContextDriver;
use super::frame::VideoFrame;
use super::hardware::{GfxBackendKind, HardwareFrame};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverFrame {
    Software(VideoFrame),
    Hardware(HardwareFrame),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentStatus {
    pub backend: GfxBackendKind,
    pub frame_number: u64,
    pub rendered: bool,
    pub details: String,
}

pub trait GfxDriver {
    fn name(&self) -> &'static str;
    fn init(&mut self, context: &ContextDriver, bootstrap_frame: &VideoFrame);
    fn context_reset(&mut self, context: &ContextDriver);
    fn present(&mut self, frame: &DriverFrame) -> Result<PresentStatus, String>;
    fn destroy(&mut self);
}
