use super::{DriverFrame, GfxDriver, PresentStatus};
use crate::gfx::context::ContextDriver;
use crate::gfx::frame::VideoFrame;
use crate::gfx::hardware::{GfxBackendKind, HostRenderHandles, VulkanRenderCommand};
use crate::gfx::CLEAR_COLOR_RGBA;
use std::ptr;

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
    pub source_is_hardware: bool,
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

    fn build_plan(
        &self,
        width: u32,
        height: u32,
        source_is_hardware: bool,
    ) -> Result<VulkanPresentPlan, String> {
        if !self.handles.has_vulkan() {
            return Err("Vulkan backend requires MoltenVK/native view, instance, device, queue, command buffer, image, and render callback handles".into());
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
            source_is_hardware,
            required_instance_extensions: &["VK_KHR_surface", "VK_EXT_metal_surface"],
            required_device_extensions: &["VK_KHR_swapchain"],
        })
    }

    fn execute(&self, plan: &VulkanPresentPlan, rgba: Option<&[u8]>) -> Result<(), String> {
        let callback = self
            .handles
            .vulkan_render
            .ok_or_else(|| "Vulkan render callback was not configured".to_string())?;
        let command = VulkanRenderCommand {
            native_view: plan.native_view,
            instance: plan.instance,
            device: plan.device,
            queue: plan.queue,
            command_buffer: plan.command_buffer,
            image: plan.image,
            extent: plan.extent,
            source_is_hardware: plan.source_is_hardware,
            uses_moltenvk: plan.uses_moltenvk,
            clear_color: CLEAR_COLOR_RGBA,
        };
        let (ptr, len) = rgba.map_or((ptr::null(), 0), |bytes| (bytes.as_ptr(), bytes.len()));
        let rendered = unsafe { callback(&command, ptr, len, self.handles.user_data) };
        if rendered {
            Ok(())
        } else {
            Err("Vulkan render callback reported failure".into())
        }
    }
}

impl GfxDriver for VulkanDriver {
    fn name(&self) -> &'static str {
        "vulkan-moltenvk-host"
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
        let (width, height, frame_number, source_is_hardware, rgba) = match frame {
            DriverFrame::Software(frame) => (
                frame.width,
                frame.height,
                0,
                false,
                Some(frame.rgba.as_slice()),
            ),
            DriverFrame::Hardware(frame) => {
                (frame.width, frame.height, frame.frame_number, true, None)
            }
        };
        let plan = self.build_plan(width, height, source_is_hardware)?;
        self.execute(&plan, rgba)?;
        self.last_present_plan = Some(plan);
        Ok(PresentStatus {
            backend: GfxBackendKind::Vulkan,
            frame_number,
            rendered: true,
            details: format!("Vulkan/MoltenVK rendered {width}x{height} through host callback"),
        })
    }
    fn destroy(&mut self) {
        self.initialized = false;
        self.last_present_plan = None;
    }
}
