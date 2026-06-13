use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{
    menu::{MenuLayout, MenuModel},
    shader::ShaderManager,
};

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
    pub driver: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderCommand {
    MenuDriver {
        name: String,
        source: String,
        layout: MenuLayout,
        accent_rgba: u32,
        background_rgba: u32,
        row_height: u32,
        icon_size: u32,
        sidebar_width: u32,
        thumbnail_size: u32,
    },
    MenuAsset {
        kind: MenuAssetKind,
        path: PathBuf,
        driver: Option<String>,
    },
    MenuTitle(String),
    MenuEntry {
        label: String,
        selected: bool,
    },
    Frame {
        width: u32,
        height: u32,
    },
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
            let driver = classify_menu_driver_asset(&path);
            self.menu_assets.push(MenuAsset {
                kind,
                name,
                path,
                driver,
            });
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
        let driver = menu.driver().descriptor();
        self.commands.push(RenderCommand::MenuDriver {
            name: driver.name.to_owned(),
            source: driver.source_file.to_owned(),
            layout: driver.layout,
            accent_rgba: driver.accent_rgba,
            background_rgba: driver.background_rgba,
            row_height: driver.row_height,
            icon_size: driver.icon_size,
            sidebar_width: driver.sidebar_width,
            thumbnail_size: driver.thumbnail_size,
        });
        for asset in self
            .menu_assets
            .iter()
            .filter(|asset| {
                asset
                    .driver
                    .as_deref()
                    .is_none_or(|name| name == driver.name)
            })
            .take(128)
        {
            self.commands.push(RenderCommand::MenuAsset {
                kind: asset.kind.clone(),
                path: asset.path.clone(),
                driver: asset.driver.clone(),
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

    /// Write a deterministic UI milestone snapshot without requiring a window.
    ///
    /// This is intentionally simple: it proves that the current menu page,
    /// selected row, driver theme colours and placeholder thumbnail area can be
    /// rendered as an image even before the real platform GPU presentation path
    /// is wired into every host shell.
    pub fn write_menu_snapshot_ppm(
        &self,
        menu: &MenuModel,
        path: impl AsRef<Path>,
    ) -> io::Result<()> {
        let width = self.frame_size.width.max(640);
        let height = self.frame_size.height.max(360);
        let driver = menu.driver().descriptor();
        let mut pixels = vec![0_u8; width as usize * height as usize * 3];
        let bg = rgba_to_rgb(driver.background_rgba);
        fill_rect(&mut pixels, width, height, 0, 0, width, height, bg);

        let accent = rgba_to_rgb(driver.accent_rgba);
        let sidebar = driver.sidebar_width.min(width / 2);
        if sidebar > 0 {
            fill_rect(
                &mut pixels,
                width,
                height,
                0,
                0,
                sidebar,
                height,
                darken(accent),
            );
        }
        fill_rect(&mut pixels, width, height, 0, 0, width, 48, darken(bg));
        draw_text_blocks(&mut pixels, width, height, 24, 16, menu.title(), accent);

        let row_height = driver.row_height.max(16);
        let start_x = sidebar.saturating_add(24).min(width.saturating_sub(120));
        let mut y = 72;
        for (index, entry) in menu.current_entries().iter().enumerate().take(12) {
            let selected = index == menu.current_selection();
            let row_color = if selected {
                accent
            } else if index % 2 == 0 {
                lighten(bg)
            } else {
                bg
            };
            fill_rect(
                &mut pixels,
                width,
                height,
                start_x.saturating_sub(8),
                y,
                width.saturating_sub(start_x + 24),
                row_height.saturating_sub(4),
                row_color,
            );
            draw_text_blocks(
                &mut pixels,
                width,
                height,
                start_x,
                y + 6,
                &entry.label,
                if selected {
                    (255, 255, 255)
                } else {
                    (220, 225, 230)
                },
            );
            if !entry.value.is_empty() {
                draw_text_blocks(
                    &mut pixels,
                    width,
                    height,
                    width.saturating_sub(220),
                    y + 6,
                    &entry.value,
                    (180, 190, 200),
                );
            }
            if !entry.sublabel.is_empty() && row_height >= 32 {
                draw_text_blocks(
                    &mut pixels,
                    width,
                    height,
                    start_x,
                    y + 24,
                    &entry.sublabel,
                    (145, 155, 165),
                );
            }
            y = y.saturating_add(row_height);
        }

        if driver.thumbnail_size > 0 {
            let size = driver.thumbnail_size.min(width / 3).min(height / 2);
            let x = width.saturating_sub(size + 32);
            let y = height.saturating_sub(size + 32);
            fill_rect(&mut pixels, width, height, x, y, size, size, darken(accent));
            fill_rect(
                &mut pixels,
                width,
                height,
                x + 8,
                y + 8,
                size.saturating_sub(16),
                size.saturating_sub(16),
                (40, 44, 52),
            );
            draw_text_blocks(
                &mut pixels,
                width,
                height,
                x + 18,
                y + 18,
                "thumbnail placeholder",
                (210, 215, 220),
            );
        }

        let mut file = fs::File::create(path)?;
        write!(file, "P6\n{} {}\n255\n", width, height)?;
        file.write_all(&pixels)?;
        Ok(())
    }
}

fn rgba_to_rgb(rgba: u32) -> (u8, u8, u8) {
    (
        ((rgba >> 24) & 0xff) as u8,
        ((rgba >> 16) & 0xff) as u8,
        ((rgba >> 8) & 0xff) as u8,
    )
}

fn darken((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    (r / 2, g / 2, b / 2)
}

fn lighten((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    (
        r.saturating_add(18),
        g.saturating_add(18),
        b.saturating_add(18),
    )
}

fn fill_rect(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: (u8, u8, u8),
) {
    let max_y = y.saturating_add(h).min(height);
    let max_x = x.saturating_add(w).min(width);
    for py in y..max_y {
        for px in x..max_x {
            let offset = ((py * width + px) * 3) as usize;
            pixels[offset] = color.0;
            pixels[offset + 1] = color.1;
            pixels[offset + 2] = color.2;
        }
    }
}

fn draw_text_blocks(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    text: &str,
    color: (u8, u8, u8),
) {
    let mut cursor = x;
    for byte in text.bytes().take(38) {
        if byte == b' ' {
            cursor = cursor.saturating_add(6);
            continue;
        }
        let block_h = 8 + u32::from(byte % 5);
        fill_rect(pixels, width, height, cursor, y, 4, block_h, color);
        cursor = cursor.saturating_add(7);
        if cursor >= width.saturating_sub(8) {
            break;
        }
    }
}

fn classify_menu_driver_asset(path: &Path) -> Option<String> {
    let mut previous_was_assets = false;
    for component in path.components() {
        let name = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        match name.as_str() {
            "ozone" | "xmb" | "rgui" => return Some(name),
            "glui" | "materialui" => return Some("materialui".to_owned()),
            _ => {}
        }
        if previous_was_assets {
            return match name.as_str() {
                "ozone" | "xmb" | "rgui" => Some(name),
                "glui" | "materialui" => Some("materialui".to_owned()),
                _ => None,
            };
        }
        previous_was_assets = name == "assets";
    }
    None
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
            .menu_assets()
            .iter()
            .any(|asset| asset.driver.as_deref() == Some("xmb")));
        assert!(renderer
            .commands()
            .iter()
            .all(|command| !matches!(command, RenderCommand::MenuTitle(_))));
        let _ = std::fs::remove_dir_all(&root);
    }
}
