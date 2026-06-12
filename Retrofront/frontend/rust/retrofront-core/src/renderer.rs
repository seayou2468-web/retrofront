use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::shader::ShaderManager;

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("window handle error: {0}")]
    WindowHandle(#[from] raw_window_handle::HandleError),
    #[error("surface creation error: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    #[error("request device error: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererBackend {
    Wgpu,
}

/// WGPU renderer facade used by menu video drawing and libretro video frames.
pub struct VideoRenderer {
    backend: RendererBackend,
    frame_size: FrameSize,
    gpu: Option<WgpuState>,
}

impl VideoRenderer {
    pub fn new() -> Self {
        Self {
            backend: RendererBackend::Wgpu,
            frame_size: FrameSize::default(),
            gpu: None,
        }
    }

    pub fn backend(&self) -> RendererBackend {
        self.backend
    }
    pub fn frame_size(&self) -> FrameSize {
        self.frame_size
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.frame_size = FrameSize { width, height };
    }

    pub async fn initialize_for_window<W>(&mut self, window: &W) -> Result<(), RendererError>
    where
        W: HasWindowHandle + HasDisplayHandle + Send + Sync,
    {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::VULKAN | wgpu::Backends::METAL;
        let instance = wgpu::Instance::new(desc);
        let surface_target =
            unsafe { wgpu::SurfaceTargetUnsafe::from_display_and_window(window, window) }
                .map_err(RendererError::WindowHandle)?;
        let surface = unsafe { instance.create_surface_unsafe(surface_target) }
            .map_err(RendererError::CreateSurface)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("wgpu adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("retrofront-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(RendererError::RequestDevice)?;
        self.gpu = Some(WgpuState {
            instance,
            adapter,
            device,
            queue,
            surface,
        });
        Ok(())
    }

    /// Draw the current menu tree.  The concrete pass construction is kept in
    /// Rust; C menu drivers should submit entries to this facade instead of
    /// depending on RetroArch renderer globals.
    pub fn draw_menu(&mut self, _shaders: &mut ShaderManager) {
        // Pipeline creation and text/thumbnail batching will live here.  This
        // method is intentionally the single Rust-owned video entry point for
        // `menu/` so future C shims remain thin.
    }

    pub fn wgpu_state(&self) -> Option<&WgpuState> {
        self.gpu.as_ref()
    }
}

impl Default for VideoRenderer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WgpuState {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
}
