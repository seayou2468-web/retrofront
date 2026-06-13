use serde::{Deserialize, Serialize};

use crate::input::MenuAction;

pub const MENU_LABEL_MAX_LENGTH: usize = 1024;

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MenuIntent {
    OpenPath(String),
    LaunchContent {
        core_path: String,
        game_path: String,
    },
    ToggleBool(String),
    Back,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuModel {
    title: String,
    stack: Vec<Vec<MenuEntry>>,
    titles: Vec<String>,
    selection: Vec<usize>,
}

impl Default for MenuModel {
    fn default() -> Self {
        Self {
            title: "Retrofront".into(),
            stack: vec![Vec::new()],
            titles: vec!["Retrofront".into()],
            selection: vec![0],
        }
    }
}

impl MenuModel {
    pub fn set_root(&mut self, title: impl Into<String>, entries: Vec<MenuEntry>) {
        self.title = title.into();
        self.stack.clear();
        self.stack.push(entries);
        self.titles.clear();
        self.titles.push(self.title.clone());
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
    pub fn selected_entry(&self) -> Option<&MenuEntry> {
        self.current_entries().get(self.current_selection())
    }
    pub fn push_with_title(&mut self, title: impl Into<String>, entries: Vec<MenuEntry>) {
        let title = title.into();
        self.title = title.clone();
        self.titles.push(title);
        self.stack.push(entries);
        self.selection.push(0);
    }
    pub fn push(&mut self, entries: Vec<MenuEntry>) {
        self.push_with_title(self.title.clone(), entries);
    }
    pub fn pop(&mut self) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop();
            self.selection.pop();
            self.titles.pop();
            self.title = self
                .titles
                .last()
                .cloned()
                .unwrap_or_else(|| "Retrofront".into());
            true
        } else {
            false
        }
    }
    pub fn action(&mut self, action: MenuAction) -> Option<MenuIntent> {
        let len = self.current_entries().len();
        let idx = self.selection.last_mut()?;
        match action {
            MenuAction::Up if len > 0 => *idx = idx.saturating_sub(1),
            MenuAction::Down if len > 0 => *idx = (*idx + 1).min(len - 1),
            MenuAction::Left if len > 0 => *idx = idx.saturating_sub(10),
            MenuAction::Right if len > 0 => *idx = (*idx + 10).min(len - 1),
            MenuAction::Cancel => return self.pop().then_some(MenuIntent::Back),
            MenuAction::Ok => {
                let entry = self.selected_entry()?.clone();
                if let Some(rest) = entry.path.strip_prefix("launch://") {
                    let (core_path, game_path) = rest.split_once('|').unwrap_or(("", rest));
                    return Some(MenuIntent::LaunchContent {
                        core_path: core_path.into(),
                        game_path: game_path.into(),
                    });
                }
                if !entry.path.is_empty() {
                    return Some(MenuIntent::OpenPath(entry.path));
                }
            }
            _ => {}
        }
        None
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
