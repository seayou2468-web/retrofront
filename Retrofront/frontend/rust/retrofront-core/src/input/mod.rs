#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    Up,
    Down,
    Left,
    Right,
    Accept,
    Back,
    Search,
    NextTab,
    PrevTab,
}

#[derive(Default, Debug)]
pub struct InputMapper {
    repeat_ms: u64,
    analog_threshold: f32,
}
impl InputMapper {
    pub fn new() -> Self {
        Self {
            repeat_ms: 140,
            analog_threshold: 0.45,
        }
    }
    pub fn repeat_ms(&self) -> u64 {
        self.repeat_ms
    }
    pub fn analog_threshold(&self) -> f32 {
        self.analog_threshold
    }
}
