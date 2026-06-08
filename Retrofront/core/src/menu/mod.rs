use crate::core_info::CoreInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntryKind {
    Action,
    Submenu,
    Toggle,
    Setting,
}

pub struct MenuEntry {
    pub label: String,
    pub sublabel: String,
    pub kind: MenuEntryKind,
    pub value: String,
    pub action_id: u32,
}

pub struct MenuList {
    pub title: String,
    pub entries: Vec<MenuEntry>,
}

pub struct MenuEngine {
    pub history: Vec<MenuList>,
}

impl MenuEngine {
    pub fn new() -> Self {
        let mut engine = Self { history: Vec::new() };
        engine.push_main_menu();
        engine
    }

    pub fn push_main_menu(&mut self) {
        let mut entries = Vec::new();
        entries.push(MenuEntry {
            label: "Load Core".to_string(),
            sublabel: "Select an emulator core to load".to_string(),
            kind: MenuEntryKind::Submenu,
            value: "".to_string(),
            action_id: 1,
        });
        entries.push(MenuEntry {
            label: "Load Content".to_string(),
            sublabel: "Select a game to play".to_string(),
            kind: MenuEntryKind::Submenu,
            value: "".to_string(),
            action_id: 2,
        });
        entries.push(MenuEntry {
            label: "Settings".to_string(),
            sublabel: "Configure Retrofront settings".to_string(),
            kind: MenuEntryKind::Submenu,
            value: "".to_string(),
            action_id: 3,
        });

        self.history.push(MenuList {
            title: "Main Menu".to_string(),
            entries,
        });
    }

    pub fn push_quick_menu(&mut self, has_game: bool) {
        let mut entries = Vec::new();
        if has_game {
            entries.push(MenuEntry {
                label: "Resume".to_string(),
                sublabel: "Continue playing the current game".to_string(),
                kind: MenuEntryKind::Action,
                value: "".to_string(),
                action_id: 10,
            });
            entries.push(MenuEntry {
                label: "Core Options".to_string(),
                sublabel: "Adjust settings for the active core".to_string(),
                kind: MenuEntryKind::Submenu,
                value: "".to_string(),
                action_id: 11,
            });
            entries.push(MenuEntry {
                label: "Close Content".to_string(),
                sublabel: "Exit the current game".to_string(),
                kind: MenuEntryKind::Action,
                value: "".to_string(),
                action_id: 12,
            });
        }

        self.history.push(MenuList {
            title: "Quick Menu".to_string(),
            entries,
        });
    }

    pub fn push_core_list(&mut self, cores: &[CoreInfo]) {
        let mut entries = Vec::new();
        for (i, core) in cores.iter().enumerate() {
            entries.push(MenuEntry {
                label: core.display_name.clone(),
                sublabel: core.system_name.clone(),
                kind: MenuEntryKind::Action,
                value: "".to_string(),
                action_id: 100 + i as u32,
            });
        }
        self.history.push(MenuList {
            title: "Load Core".to_string(),
            entries,
        });
    }

    pub fn pop(&mut self) -> Option<MenuList> {
        if self.history.len() > 1 {
            self.history.pop()
        } else {
            None
        }
    }

    pub fn current(&self) -> Option<&MenuList> {
        self.history.last()
    }

    pub fn clear_to_main(&mut self) {
        self.history.truncate(1);
    }
}
