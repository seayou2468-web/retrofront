use serde::{Deserialize, Serialize};

use crate::input::MenuAction;

pub const MENU_LABEL_MAX_LENGTH: usize = 1024;

/// Menu entry model equivalent to `menu_entry_t` without raw C buffers.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuEntry {
    pub path: String,
    pub label: String,
    pub rich_label: String,
    pub sublabel: String,
    pub value: String,
    pub entry_type: MenuEntryType,
    pub checked: bool,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum MenuEntryType {
    #[default]
    Action,
    Bool,
    Int,
    UInt,
    Float,
    Path,
    Dir,
    String,
    Hex,
    Bind,
    Enum,
}

/// In-memory menu stack and current selection requested by `menu/`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuModel {
    title: String,
    stack: Vec<Vec<MenuEntry>>,
    selection: Vec<usize>,
}

impl Default for MenuModel {
    fn default() -> Self {
        Self {
            title: "Retrofront".into(),
            stack: vec![Vec::new()],
            selection: vec![0],
        }
    }
}

impl MenuModel {
    pub fn set_root(&mut self, title: impl Into<String>, entries: Vec<MenuEntry>) {
        self.title = title.into();
        self.stack.clear();
        self.stack.push(entries);
        self.selection.clear();
        self.selection.push(0);
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn current_entries(&self) -> &[MenuEntry] {
        self.stack.last().map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn current_selection(&self) -> usize {
        *self.selection.last().unwrap_or(&0)
    }

    pub fn push(&mut self, entries: Vec<MenuEntry>) {
        self.stack.push(entries);
        self.selection.push(0);
    }

    pub fn pop(&mut self) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop();
            self.selection.pop();
            true
        } else {
            false
        }
    }

    pub fn action(&mut self, action: MenuAction) -> Option<&MenuEntry> {
        let len = self.current_entries().len();
        let idx = self.selection.last_mut()?;
        match action {
            MenuAction::Up if len > 0 => *idx = idx.saturating_sub(1),
            MenuAction::Down if len > 0 => *idx = (*idx + 1).min(len - 1),
            MenuAction::Cancel => {
                self.pop();
            }
            _ => {}
        }
        self.current_entries().get(self.current_selection())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_is_clamped() {
        let mut menu = MenuModel::default();
        menu.set_root(
            "root",
            vec![MenuEntry {
                label: "one".into(),
                ..Default::default()
            }],
        );
        menu.action(MenuAction::Down);
        assert_eq!(menu.current_selection(), 0);
    }
}
