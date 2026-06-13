#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    FillRect {
        rect: [f32; 4],
        color: [f32; 4],
    },
    DrawTexture {
        texture: u64,
        rect: [f32; 4],
        tint: [f32; 4],
    },
    DrawText {
        text: String,
        pos: [f32; 2],
        size: f32,
        color: [f32; 4],
    },
    SetClip([f32; 4]),
    ClearClip,
    PushTransform([[f32; 4]; 4]),
    PopTransform,
}
#[derive(Default, Debug)]
pub struct CommandBuffer {
    pub commands: Vec<DrawCommand>,
    pub warnings: Vec<String>,
    pub max: usize,
}
impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            warnings: Vec::new(),
            max: 16384,
        }
    }
    pub fn append(&mut self, c: DrawCommand) {
        if self.commands.len() < self.max {
            self.commands.push(c)
        } else {
            self.warnings.push("draw command buffer overflow".into())
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let mut stack = 0;
        for c in &self.commands {
            match c {
                DrawCommand::PushTransform(_) => stack += 1,
                DrawCommand::PopTransform => {
                    if stack == 0 {
                        return Err("transform stack underflow".into());
                    }
                    stack -= 1
                }
                _ => {}
            }
        }
        if stack == 0 {
            Ok(())
        } else {
            Err("unbalanced transform stack".into())
        }
    }
}
