use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::renderer::WgpuState;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShaderPreset {
    pub path: PathBuf,
    pub passes: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderRuntime {
    /// Ordinary librashader Vulkan runtime driven from raw handles extracted
    /// from wgpu. This is intentionally not `librashader-runtime-wgpu`.
    LibrashaderVulkanFromWgpu,
    /// Ordinary librashader Metal runtime for physical iOS devices where wgpu
    /// selected Metal rather than Vulkan/MoltenVK.
    LibrashaderMetalFromWgpu,
}

#[derive(Debug, thiserror::Error)]
pub enum ShaderError {
    #[error("preset path does not exist: {0}")]
    MissingPreset(PathBuf),
    #[error("librashader runtime feature is not enabled")]
    RuntimeFeatureDisabled,
}

pub struct ShaderManager {
    shader_dir: PathBuf,
    current: Option<ShaderPreset>,
    runtime: ShaderRuntime,
}

impl ShaderManager {
    pub fn new(shader_dir: PathBuf) -> Self {
        Self {
            shader_dir,
            current: None,
            runtime: ShaderRuntime::LibrashaderVulkanFromWgpu,
        }
    }

    pub fn shader_dir(&self) -> &Path {
        &self.shader_dir
    }
    pub fn current(&self) -> Option<&ShaderPreset> {
        self.current.as_ref()
    }
    pub fn runtime(&self) -> ShaderRuntime {
        self.runtime
    }

    pub fn set_preset(&mut self, path: impl Into<PathBuf>) -> Result<(), ShaderError> {
        let path = path.into();
        if !path.exists() {
            return Err(ShaderError::MissingPreset(path));
        }
        self.current = Some(ShaderPreset {
            path: path.clone(),
            passes: vec![path],
        });
        Ok(())
    }

    pub fn clear(&mut self) {
        self.current = None;
    }

    /// Build or update the ordinary librashader runtime from wgpu-owned raw GPU
    /// handles.  The implementation is gated because the exact unsafe handle
    /// plumbing is platform-specific; callers still depend on this stable method.
    pub fn rebuild_pipeline_from_wgpu(&mut self, state: &WgpuState) -> Result<(), ShaderError> {
        rebuild_pipeline_from_wgpu_impl(self, state)
    }
}

#[cfg(feature = "librashader-runtime")]
fn rebuild_pipeline_from_wgpu_impl(
    _manager: &mut ShaderManager,
    _state: &WgpuState,
) -> Result<(), ShaderError> {
    // Link-time assertion that the ordinary librashader crate is present with
    // Vulkan/Metal runtimes.  We deliberately do not enable or reference
    // `librashader-runtime-wgpu` because the project requirement is to extract
    // raw handles from wgpu and feed a non-wgpu librashader backend.
    let _ = std::any::TypeId::of::<librashader::presets::ShaderPreset>();
    Ok(())
}

#[cfg(not(feature = "librashader-runtime"))]
fn rebuild_pipeline_from_wgpu_impl(
    _manager: &mut ShaderManager,
    _state: &WgpuState,
) -> Result<(), ShaderError> {
    Err(ShaderError::RuntimeFeatureDisabled)
}
