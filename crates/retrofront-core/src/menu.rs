use std::{fs, path::Path};

#[derive(Clone, Debug, PartialEq)]
pub struct SkinTheme {
    pub name: String,
    pub font: String,
    pub background: [f32; 4],
    pub foreground: [f32; 4],
    pub accent: [f32; 4],
    pub icon_set: String,
}

impl Default for SkinTheme {
    fn default() -> Self {
        Self {
            name: "RetroFront XMB".into(),
            font: "Inter".into(),
            background: [0.03, 0.04, 0.08, 1.0],
            foreground: [0.92, 0.94, 1.0, 1.0],
            accent: [0.15, 0.55, 1.0, 1.0],
            icon_set: "monochrome".into(),
        }
    }
}

impl SkinTheme {
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut theme = SkinTheme::default();
        let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            match key {
                "name" => theme.name = value.into(),
                "font" => theme.font = value.into(),
                "icon_set" => theme.icon_set = value.into(),
                "background" => theme.background = parse_rgba(value)?,
                "foreground" => theme.foreground = parse_rgba(value)?,
                "accent" => theme.accent = parse_rgba(value)?,
                _ => {}
            }
        }
        Ok(theme)
    }
}

fn parse_rgba(value: &str) -> Result<[f32; 4], String> {
    let parts = value
        .split(',')
        .map(|p| p.trim().parse::<f32>().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() != 4 {
        return Err(format!("expected rgba tuple, got {value}"));
    }
    Ok([parts[0], parts[1], parts[2], parts[3]])
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuItem {
    LoadCore,
    LoadContent,
    QuickMenu,
    Settings,
    History,
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuModel {
    pub theme: SkinTheme,
    pub items: Vec<MenuItem>,
    pub selected: usize,
}

impl MenuModel {
    pub fn xmb(theme: SkinTheme) -> Self {
        Self {
            theme,
            items: vec![
                MenuItem::LoadCore,
                MenuItem::LoadContent,
                MenuItem::QuickMenu,
                MenuItem::Settings,
                MenuItem::History,
                MenuItem::Quit,
            ],
            selected: 0,
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.items.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
    }

    pub fn selected_item(&self) -> &MenuItem {
        &self.items[self.selected]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wraps_menu_selection() {
        let mut model = MenuModel::xmb(SkinTheme::default());
        model.move_selection(-1);
        assert_eq!(model.selected_item(), &MenuItem::Quit);
    }
}
