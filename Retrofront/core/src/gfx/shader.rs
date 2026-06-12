use super::hardware::{GfxBackendKind, HostRenderHandles};
use librashader::presets::{ShaderFeatures, ShaderPreset};
use std::path::{Path, PathBuf};

/// librashader runtime family selected from the native graphics handles.
///
/// This intentionally omits the librashader wgpu runtime. The host may still render through the
/// existing wgpu callback, but shader execution is prepared against the raw Metal/OpenGL/Vulkan
/// handles that are carried through [`HostRenderHandles`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibrashaderRuntimeKind {
    Metal,
    OpenGl,
    Vulkan,
}

impl LibrashaderRuntimeKind {
    pub fn from_backend(backend: GfxBackendKind) -> Option<Self> {
        match backend {
            GfxBackendKind::Metal | GfxBackendKind::MoltenVk => Some(Self::Metal),
            GfxBackendKind::OpenGl => Some(Self::OpenGl),
            GfxBackendKind::Vulkan => Some(Self::Vulkan),
            GfxBackendKind::Software | GfxBackendKind::Wgpu => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Metal => "librashader-metal",
            Self::OpenGl => "librashader-gl",
            Self::Vulkan => "librashader-vulkan",
        }
    }
}

/// Native handles extracted from the host renderer before creating a concrete librashader filter
/// chain. The actual runtime constructors need platform-specific objects, so this bridge keeps the
/// raw values close to the gfx backend and prevents the UI/runtime model from owning shader details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RawLibrashaderHandles {
    pub native_view: u64,
    pub context: u64,
    pub framebuffer: usize,
}

impl From<HostRenderHandles> for RawLibrashaderHandles {
    fn from(handles: HostRenderHandles) -> Self {
        Self {
            native_view: handles.native_view,
            context: handles.context,
            framebuffer: handles.framebuffer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibrashaderStatus {
    pub preset_path: Option<PathBuf>,
    pub runtime: Option<LibrashaderRuntimeKind>,
    pub pass_count: usize,
    pub parameter_count: usize,
    pub ready: bool,
    pub message: String,
}

impl Default for LibrashaderStatus {
    fn default() -> Self {
        Self {
            preset_path: None,
            runtime: None,
            pass_count: 0,
            parameter_count: 0,
            ready: false,
            message: "No shader preset loaded".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LibrashaderPipeline {
    preset: Option<ShaderPreset>,
    preset_path: Option<PathBuf>,
    runtime: Option<LibrashaderRuntimeKind>,
    raw_handles: RawLibrashaderHandles,
    status: LibrashaderStatus,
}

impl Default for LibrashaderPipeline {
    fn default() -> Self {
        Self {
            preset: None,
            preset_path: None,
            runtime: None,
            raw_handles: RawLibrashaderHandles::default(),
            status: LibrashaderStatus::default(),
        }
    }
}

impl LibrashaderPipeline {
    pub fn configure_from_host(&mut self, backend: GfxBackendKind, handles: HostRenderHandles) {
        self.runtime = LibrashaderRuntimeKind::from_backend(backend);
        self.raw_handles = handles.into();
        self.refresh_status();
    }

    pub fn load_preset(&mut self, path: impl AsRef<Path>) -> Result<LibrashaderStatus, String> {
        let path = path.as_ref().to_path_buf();
        let preset = ShaderPreset::try_parse(&path, ShaderFeatures::empty()).map_err(|error| {
            format!("failed to parse shader preset {}: {error}", path.display())
        })?;
        self.preset = Some(preset);
        self.preset_path = Some(path);
        self.refresh_status();
        Ok(self.status.clone())
    }

    pub fn clear(&mut self) {
        self.preset = None;
        self.preset_path = None;
        self.status = LibrashaderStatus::default();
    }

    pub fn status(&self) -> &LibrashaderStatus {
        &self.status
    }

    pub fn has_usable_raw_handles(&self) -> bool {
        self.raw_handles.native_view != 0
            || self.raw_handles.context != 0
            || self.raw_handles.framebuffer != 0
    }

    fn refresh_status(&mut self) {
        let pass_count = self
            .preset
            .as_ref()
            .map(|preset| preset.pass_count.max(0) as usize)
            .unwrap_or_default();
        let parameter_count = self
            .preset
            .as_ref()
            .map(|preset| preset.parameters.len())
            .unwrap_or_default();
        let ready =
            self.preset.is_some() && self.runtime.is_some() && self.has_usable_raw_handles();
        let message = match (
            self.preset.is_some(),
            self.runtime,
            self.has_usable_raw_handles(),
        ) {
            (false, _, _) => "No shader preset loaded".to_string(),
            (true, None, _) => {
                "Shader preset parsed; waiting for Metal/OpenGL/Vulkan backend".to_string()
            }
            (true, Some(runtime), false) => format!(
                "Shader preset parsed for {}; waiting for raw host handles",
                runtime.label()
            ),
            (true, Some(runtime), true) => format!(
                "Shader preset ready for {} with {pass_count} passes",
                runtime.label()
            ),
        };
        self.status = LibrashaderStatus {
            preset_path: self.preset_path.clone(),
            runtime: self.runtime,
            pass_count,
            parameter_count,
            ready,
            message,
        };
    }
}
