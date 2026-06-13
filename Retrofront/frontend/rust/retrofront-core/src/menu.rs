use serde::{Deserialize, Serialize};

use crate::input::MenuAction;

pub const MENU_LABEL_MAX_LENGTH: usize = 1024;
include!(concat!(env!("OUT_DIR"), "/menu_sources.rs"));

#[repr(C)]
struct CMenuDriverDescriptor {
    name: *const std::ffi::c_char,
    source_file: *const std::ffi::c_char,
    layout: u32,
    accent_rgba: u32,
    background_rgba: u32,
    row_height: u32,
    icon_size: u32,
    sidebar_width: u32,
    thumbnail_size: u32,
}

extern "C" {
    fn retrofront_c_menu_driver_by_name(
        name: *const std::ffi::c_char,
    ) -> *const CMenuDriverDescriptor;
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum MenuLayout {
    #[default]
    Ozone = 1,
    Xmb = 2,
    MaterialUi = 3,
    Rgui = 4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuDriverDescriptor {
    pub name: &'static str,
    pub source_file: &'static str,
    pub layout: MenuLayout,
    pub accent_rgba: u32,
    pub background_rgba: u32,
    pub row_height: u32,
    pub icon_size: u32,
    pub sidebar_width: u32,
    pub thumbnail_size: u32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum MenuDriver {
    Ozone,
    Xmb,
    #[default]
    MaterialUi,
    Rgui,
}

impl MenuDriver {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ozone" => Some(Self::Ozone),
            "xmb" => Some(Self::Xmb),
            "materialui" => Some(Self::MaterialUi),
            "rgui" => Some(Self::Rgui),
            _ => None,
        }
    }

    pub fn as_name(self) -> &'static str {
        match self {
            Self::Ozone => "ozone",
            Self::Xmb => "xmb",
            Self::MaterialUi => "materialui",
            Self::Rgui => "rgui",
        }
    }

    pub fn source_file(self) -> &'static str {
        match self {
            Self::Ozone => "drivers/ozone.c",
            Self::Xmb => "drivers/xmb.c",
            Self::MaterialUi => "drivers/materialui.c",
            Self::Rgui => "drivers/rgui.c",
        }
    }

    pub fn descriptor(self) -> MenuDriverDescriptor {
        let cname = std::ffi::CString::new(self.as_name()).expect("static driver name has no nul");
        let ptr = unsafe { retrofront_c_menu_driver_by_name(cname.as_ptr()) };
        if !ptr.is_null() {
            let c = unsafe { &*ptr };
            return MenuDriverDescriptor {
                name: self.as_name(),
                source_file: self.source_file(),
                layout: match c.layout {
                    2 => MenuLayout::Xmb,
                    3 => MenuLayout::MaterialUi,
                    4 => MenuLayout::Rgui,
                    _ => MenuLayout::Ozone,
                },
                accent_rgba: c.accent_rgba,
                background_rgba: c.background_rgba,
                row_height: c.row_height,
                icon_size: c.icon_size,
                sidebar_width: c.sidebar_width,
                thumbnail_size: c.thumbnail_size,
            };
        }
        MenuDriverDescriptor {
            name: self.as_name(),
            source_file: self.source_file(),
            layout: MenuLayout::default(),
            accent_rgba: 0xffffffff,
            background_rgba: 0x000000ff,
            row_height: 48,
            icon_size: 32,
            sidebar_width: 0,
            thumbnail_size: 0,
        }
    }
}

pub const FIXED_MENU_DRIVERS: &[MenuDriver] = &[
    MenuDriver::Ozone,
    MenuDriver::Xmb,
    MenuDriver::MaterialUi,
    MenuDriver::Rgui,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedMenuSource {
    pub path: &'static str,
    pub is_driver: bool,
}

pub fn fixed_menu_sources() -> impl Iterator<Item = FixedMenuSource> {
    RETROFRONT_MENU_SOURCE_FILES
        .iter()
        .copied()
        .map(|path| FixedMenuSource {
            path,
            is_driver: FIXED_MENU_DRIVERS
                .iter()
                .any(|driver| driver.source_file() == path),
        })
}

pub fn fixed_menu_contract_complete() -> bool {
    RETROFRONT_MENU_SOURCE_FILES.len() == 34
        && FIXED_MENU_DRIVERS
            .iter()
            .all(|driver| RETROFRONT_MENU_SOURCE_FILES.contains(&driver.source_file()))
}

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

impl MenuEntryType {
    pub fn as_u32(self) -> u32 {
        match self {
            Self::Action => 0,
            Self::Bool => 1,
            Self::Int => 2,
            Self::UInt => 3,
            Self::Float => 4,
            Self::Path => 5,
            Self::Dir => 6,
            Self::String => 7,
            Self::Hex => 8,
            Self::Bind => 9,
            Self::Enum => 10,
        }
    }
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
    driver: MenuDriver,
    stack: Vec<Vec<MenuEntry>>,
    titles: Vec<String>,
    selection: Vec<usize>,
}

impl Default for MenuModel {
    fn default() -> Self {
        Self {
            title: "Retrofront".into(),
            driver: MenuDriver::default(),
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
    pub fn driver(&self) -> MenuDriver {
        self.driver
    }
    pub fn set_driver(&mut self, driver: MenuDriver) {
        self.driver = driver;
    }
    pub fn current_entries(&self) -> &[MenuEntry] {
        self.stack.last().map(Vec::as_slice).unwrap_or(&[])
    }
    pub fn current_selection(&self) -> usize {
        *self.selection.last().unwrap_or(&0)
    }
    pub fn set_selection(&mut self, index: usize) {
        let max = self.current_entries().len().saturating_sub(1);
        if let Some(selection) = self.selection.last_mut() {
            *selection = index.min(max);
        }
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
    #[test]
    fn fixed_c_menu_contract_contains_all_34_files_and_compiled_bridge() {
        assert!(fixed_menu_contract_complete());
        assert_eq!(fixed_menu_sources().count(), 34);
        assert!(RETROFRONT_MENU_SOURCE_FILES.contains(&"drivers/ozone.c"));
        assert!(RETROFRONT_MENU_SOURCE_FILES.contains(&"drivers/materialui.c"));
        assert!(RETROFRONT_MENU_SOURCE_FILES.contains(&"drivers/xmb.c"));
        assert!(RETROFRONT_MENU_SOURCE_FILES.contains(&"drivers/rgui.c"));
        assert!(RETROFRONT_MENU_SOURCE_FILES.contains(&"retrofront_menu_bridge.c"));
        assert_eq!(MenuDriver::Ozone.descriptor().layout, MenuLayout::Ozone);
        assert_eq!(MenuDriver::Xmb.descriptor().layout, MenuLayout::Xmb);
        assert_eq!(
            MenuDriver::MaterialUi.descriptor().layout,
            MenuLayout::MaterialUi
        );
        assert_eq!(MenuDriver::Rgui.descriptor().layout, MenuLayout::Rgui);
    }
}
