#[derive(Clone, Copy, Debug, Default)]
pub struct SurfaceState {
    pub width: u32,
    pub height: u32,
    pub dpi: f32,
    pub safe_area: [f32; 4],
}
impl SurfaceState {
    pub fn resize(&mut self, w: u32, h: u32, dpi: f32) {
        self.width = w.max(1);
        self.height = h.max(1);
        self.dpi = dpi.max(0.25);
    }
}
