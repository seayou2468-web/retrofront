use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MenuAction {
    Up,
    Down,
    Left,
    Right,
    Ok,
    Cancel,
    Start,
    Select,
    Info,
    Scan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputSource {
    Key(u32),
    GamepadButton { port: u8, id: u16 },
    Touch { id: u64 },
}

#[derive(Clone, Debug)]
pub struct InputEvent {
    pub source: InputSource,
    pub pressed: bool,
}

#[derive(Default)]
pub struct InputSystem {
    bindings: HashMap<InputSource, MenuAction>,
    events: VecDeque<MenuAction>,
}

impl InputSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, source: InputSource, action: MenuAction) {
        self.bindings.insert(source, action);
    }

    pub fn push_event(&mut self, event: InputEvent) {
        if event.pressed {
            if let Some(action) = self.bindings.get(&event.source) {
                self.events.push_back(*action);
            }
        }
    }

    pub fn next_action(&mut self) -> Option<MenuAction> {
        self.events.pop_front()
    }

    pub fn begin_frame(&mut self) {}
}
