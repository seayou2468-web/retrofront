use std::collections::{HashMap, HashSet, VecDeque};

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
    pressed: HashSet<InputSource>,
    analog: HashMap<(u8, u32, u32), i16>,
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
            self.pressed.insert(event.source);
            if let Some(action) = self.bindings.get(&event.source) {
                self.events.push_back(*action);
            }
        } else {
            self.pressed.remove(&event.source);
        }
    }

    pub fn next_action(&mut self) -> Option<MenuAction> {
        self.events.pop_front()
    }

    pub fn set_analog(&mut self, port: u8, device: u32, index: u32, value: i16) {
        self.analog.insert((port, device, index), value);
    }

    pub fn libretro_button_state(&self, port: u8, id: u16) -> i16 {
        self.pressed
            .contains(&InputSource::GamepadButton { port, id })
            .then_some(1)
            .unwrap_or(0)
    }

    pub fn libretro_analog_state(&self, port: u8, device: u32, index: u32) -> i16 {
        *self.analog.get(&(port, device, index)).unwrap_or(&0)
    }

    pub fn begin_frame(&mut self) {}
}
