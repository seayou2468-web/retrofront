/// How the frontend should scale a core frame into the output surface.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GfxScaleMode {
    Stretch = 0,
    KeepAspect = 1,
    Integer = 2,
}

/// Texture sampling requested for the final frame pass.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GfxFilterMode {
    Nearest = 0,
    Linear = 1,
}

/// Runtime video geometry and presentation options mirrored from RetroArch's
/// video driver state: core geometry, output dimensions, aspect handling,
/// integer scaling, rotation, filtering and vsync.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GfxVideoConfig {
    pub base_width: u32,
    pub base_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub aspect_ratio: f32,
    pub output_width: u32,
    pub output_height: u32,
    pub scale_mode: GfxScaleMode,
    pub filter_mode: GfxFilterMode,
    pub rotation_quarters: u32,
    pub vsync: bool,
}

impl Default for GfxVideoConfig {
    fn default() -> Self {
        Self {
            base_width: 0,
            base_height: 0,
            max_width: 0,
            max_height: 0,
            aspect_ratio: 0.0,
            output_width: 0,
            output_height: 0,
            scale_mode: GfxScaleMode::KeepAspect,
            filter_mode: GfxFilterMode::Nearest,
            rotation_quarters: 0,
            vsync: true,
        }
    }
}

impl GfxVideoConfig {
    pub fn from_libretro_geometry(geometry: &crate::libretro::retro_game_geometry) -> Self {
        Self {
            base_width: geometry.base_width,
            base_height: geometry.base_height,
            max_width: geometry.max_width,
            max_height: geometry.max_height,
            aspect_ratio: geometry.aspect_ratio,
            output_width: geometry.base_width,
            output_height: geometry.base_height,
            ..Self::default()
        }
    }

    pub fn with_output_size(mut self, width: u32, height: u32) -> Self {
        self.output_width = width;
        self.output_height = height;
        self
    }

    pub fn source_aspect(self, width: u32, height: u32) -> f32 {
        if self.aspect_ratio.is_finite() && self.aspect_ratio > 0.0 {
            self.aspect_ratio
        } else if height != 0 {
            width as f32 / height as f32
        } else {
            1.0
        }
    }

    pub fn output_size(self, width: u32, height: u32) -> [u32; 2] {
        [
            self.output_width.max(width).max(1),
            self.output_height.max(height).max(1),
        ]
    }

    pub fn viewport(self, width: u32, height: u32) -> [i32; 4] {
        let [out_w, out_h] = self.output_size(width, height);
        match self.scale_mode {
            GfxScaleMode::Stretch => [0, 0, out_w as i32, out_h as i32],
            GfxScaleMode::Integer => {
                let scale = (out_w / width.max(1)).min(out_h / height.max(1)).max(1);
                let vp_w = width.max(1) * scale;
                let vp_h = height.max(1) * scale;
                [
                    ((out_w - vp_w) / 2) as i32,
                    ((out_h - vp_h) / 2) as i32,
                    vp_w as i32,
                    vp_h as i32,
                ]
            }
            GfxScaleMode::KeepAspect => {
                let aspect = self.source_aspect(width.max(1), height.max(1));
                let out_aspect = out_w as f32 / out_h as f32;
                let (vp_w, vp_h) = if out_aspect > aspect {
                    ((out_h as f32 * aspect).round() as u32, out_h)
                } else {
                    (out_w, (out_w as f32 / aspect).round() as u32)
                };
                [
                    ((out_w - vp_w) / 2) as i32,
                    ((out_h - vp_h) / 2) as i32,
                    vp_w as i32,
                    vp_h as i32,
                ]
            }
        }
    }
}
