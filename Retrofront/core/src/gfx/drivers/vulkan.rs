use super::{DriverFrame, GfxDriver, PresentStatus};
use crate::gfx::context::ContextDriver;
use crate::gfx::frame::VideoFrame;
use crate::gfx::hardware::{GfxBackendKind, HostRenderHandles};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VulkanPresentPlan {
    pub extent: [u32; 2],
    pub native_view: u64,
    pub instance: u64,
    pub device: u64,
    pub queue: u64,
    pub command_buffer: u64,
    pub image: u64,
    pub uses_moltenvk: bool,
    pub required_instance_extensions: &'static [&'static str],
    pub required_device_extensions: &'static [&'static str],
}

#[derive(Debug, Clone, Default)]
pub struct VulkanDriver {
    handles: HostRenderHandles,
    last_present_plan: Option<VulkanPresentPlan>,
    initialized: bool,
}

impl VulkanDriver {
    pub fn configure(&mut self, context: &ContextDriver) {
        self.handles = context.handles();
    }
    pub fn last_present_plan(&self) -> Option<&VulkanPresentPlan> {
        self.last_present_plan.as_ref()
    }

    fn build_plan(&self, width: u32, height: u32) -> Result<VulkanPresentPlan, String> {
        if !self.handles.has_vulkan() {
            return Err("Vulkan backend requires MoltenVK/native view, instance, device, queue, command buffer, and image handles".into());
        }
        Ok(VulkanPresentPlan {
            extent: [width, height],
            native_view: self.handles.native_view,
            instance: self.handles.vulkan_instance,
            device: self.handles.vulkan_device,
            queue: self.handles.vulkan_queue,
            command_buffer: self.handles.vulkan_command_buffer,
            image: self.handles.vulkan_image,
            uses_moltenvk: cfg!(any(target_os = "ios", target_os = "macos"))
                || self.handles.native_view != 0,
            required_instance_extensions: &["VK_KHR_surface", "VK_EXT_metal_surface"],
            required_device_extensions: &["VK_KHR_swapchain"],
        })
    }
}

impl GfxDriver for VulkanDriver {
    fn name(&self) -> &'static str {
        "vulkan-moltenvk"
    }
    fn init(&mut self, context: &ContextDriver, _bootstrap_frame: &VideoFrame) {
        self.configure(context);
        self.initialized = true;
    }
    fn context_reset(&mut self, context: &ContextDriver) {
        self.configure(context);
        self.last_present_plan = None;
    }
    fn present(&mut self, frame: &DriverFrame) -> Result<PresentStatus, String> {
        if !self.initialized {
            self.initialized = true;
        }
        let (width, height, frame_number) = match frame {
            DriverFrame::Software(frame) => (frame.width, frame.height, 0),
            DriverFrame::Hardware(frame) => (frame.width, frame.height, frame.frame_number),
        };
        let plan = self.build_plan(width, height)?;
        self.last_present_plan = Some(plan);
        Ok(PresentStatus {
            backend: GfxBackendKind::Vulkan,
            frame_number,
            rendered: true,
            details: format!("Vulkan/MoltenVK present plan prepared for {width}x{height}"),
        })
    }
    fn destroy(&mut self) {
        self.initialized = false;
        self.last_present_plan = None;
    }
}
