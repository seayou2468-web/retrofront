use std::{
    fs,
    path::{Path, PathBuf},
};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{menu::MenuModel, shader::ShaderManager};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Xrgb8888,
    Rgb565,
    Unknown(u32),
}

#[derive(Clone, Debug)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub pitch: usize,
    pub format: PixelFormat,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontAsset {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayAsset {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuAssetKind {
    Font,
    Image,
    Config,
}

impl MenuAssetKind {
    pub fn as_u32(&self) -> u32 {
        match self {
            Self::Font => 1,
            Self::Image => 2,
            Self::Config => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuAsset {
    pub kind: MenuAssetKind,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderCommand {
    MenuDriver { name: String, source: String },
    MenuAsset { kind: MenuAssetKind, path: PathBuf },
    MenuTitle(String),
    MenuEntry { label: String, selected: bool },
    Frame { width: u32, height: u32 },
}

/// WGPU renderer facade used by menu video drawing and libretro video frames.
pub struct VideoRenderer {
    backend: RendererBackend,
    frame_size: FrameSize,
    gpu: Option<WgpuState>,
    last_frame: Option<VideoFrame>,
    commands: Vec<RenderCommand>,
    fonts: Vec<FontAsset>,
    overlays: Vec<OverlayAsset>,
    menu_assets: Vec<MenuAsset>,
}

impl VideoRenderer {
    pub fn new() -> Self {
        Self {
            backend: RendererBackend::Wgpu,
            frame_size: FrameSize::default(),
            gpu: None,
            last_frame: None,
            commands: Vec::new(),
            fonts: Vec::new(),
            overlays: Vec::new(),
            menu_assets: Vec::new(),
        }
    }

    pub fn backend(&self) -> RendererBackend {
        self.backend
    }
    pub fn frame_size(&self) -> FrameSize {
        self.frame_size
    }
    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }
    pub fn last_frame(&self) -> Option<&VideoFrame> {
        self.last_frame.as_ref()
    }
    pub fn menu_assets(&self) -> &[MenuAsset] {
        &self.menu_assets
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

    pub fn load_font(&mut self, name: impl Into<String>, path: impl Into<PathBuf>) {
        self.fonts.push(FontAsset {
            name: name.into(),
            path: path.into(),
        });
    }

    pub fn load_overlay(&mut self, name: impl Into<String>, path: impl Into<PathBuf>) {
        self.overlays.push(OverlayAsset {
            name: name.into(),
            path: path.into(),
        });
    }

    pub fn load_menu_assets_from(&mut self, root: impl AsRef<Path>) -> usize {
        let before = self.menu_assets.len();
        self.scan_menu_assets(root.as_ref());
        self.menu_assets.sort_by(|a, b| a.path.cmp(&b.path));
        self.menu_assets.dedup_by(|a, b| a.path == b.path);
        self.menu_assets.len().saturating_sub(before)
    }

    fn scan_menu_assets(&mut self, dir: &Path) {
        let Ok(read_dir) = fs::read_dir(dir) else {
            return;
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_menu_assets(&path);
                continue;
            }
            let Some(ext) = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
            else {
                continue;
            };
            let kind = match ext.as_str() {
                "ttf" | "otf" | "ttc" => MenuAssetKind::Font,
                "png" | "jpg" | "jpeg" | "bmp" | "tga" => MenuAssetKind::Image,
                "cfg" | "json" | "slangp" | "glslp" | "cgp" => MenuAssetKind::Config,
                _ => continue,
            };
            if path.components().any(|c| c.as_os_str() == "__MACOSX") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("asset")
                .to_owned();
            match kind {
                MenuAssetKind::Font => self.load_font(name.clone(), &path),
                MenuAssetKind::Image | MenuAssetKind::Config => {
                    self.load_overlay(name.clone(), &path)
                }
            }
            self.menu_assets.push(MenuAsset { kind, name, path });
        }
    }

    pub fn submit_libretro_frame(&mut self, frame: VideoFrame) {
        self.commands.push(RenderCommand::Frame {
            width: frame.width,
            height: frame.height,
        });
        self.last_frame = Some(frame);
    }

    /// Draw the current menu tree.  The concrete GPU pass is Rust-owned; C menu
    /// drivers submit menu state to this facade instead of RetroArch globals.
    pub fn draw_menu(&mut self, menu: &MenuModel, shaders: &mut ShaderManager) {
        if let Some(gpu) = &self.gpu {
            let _ = shaders.rebuild_pipeline_from_wgpu(gpu);
        }
        self.commands.clear();
        self.commands.push(RenderCommand::MenuDriver {
            name: menu.driver().as_name().to_owned(),
            source: menu.driver().source_file().to_owned(),
        });
        for asset in self.menu_assets.iter().take(32) {
            self.commands.push(RenderCommand::MenuAsset {
                kind: asset.kind.clone(),
                path: asset.path.clone(),
            });
        }
        self.commands
            .push(RenderCommand::MenuTitle(menu.title().to_owned()));
        let selected = menu.current_selection();
        for (index, entry) in menu.current_entries().iter().enumerate() {
            self.commands.push(RenderCommand::MenuEntry {
                label: entry.label.clone(),
                selected: index == selected,
            });
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_assets_are_loaded_recursively_for_c_menu_drivers() {
        let root =
            std::env::temp_dir().join(format!("retrofront-menu-assets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("xmb/png")).unwrap();
        std::fs::create_dir_all(root.join("fonts")).unwrap();
        std::fs::write(root.join("xmb/png/main-menu.png"), b"png").unwrap();
        std::fs::write(root.join("fonts/menu.ttf"), b"font").unwrap();

        let mut renderer = VideoRenderer::new();
        assert_eq!(renderer.load_menu_assets_from(&root), 2);
        assert_eq!(renderer.menu_assets().len(), 2);
        assert!(renderer
            .commands()
            .iter()
            .all(|command| !matches!(command, RenderCommand::MenuTitle(_))));
        let _ = std::fs::remove_dir_all(&root);
    }
}
