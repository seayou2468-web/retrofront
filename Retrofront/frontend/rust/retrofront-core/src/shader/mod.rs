#[derive(Clone, Debug, Default)]
pub struct LibrashaderBoundary {
    pub backend: &'static str,
    pub raw_handle_ready: bool,
    pub active_preset: Option<String>,
}
impl LibrashaderBoundary {
    pub fn wgpu_raw_handle_placeholder() -> Self {
        Self {
            backend: "wgpu/raw-window-handle",
            raw_handle_ready: false,
            active_preset: Some("crt-royale.slangp".into()),
        }
    }
    pub fn preview_label(&self) -> String {
        format!(
            "{} preview via {}",
            self.active_preset.as_deref().unwrap_or("none"),
            self.backend
        )
    }
}
