use std::{fs, path::Path};

use crate::libretro::*;

#[derive(Clone, Debug, PartialEq)]
pub enum OverlayHitShape {
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControllerSkinButton {
    pub name: String,
    pub port: u32,
    pub retro_id: u32,
    pub shape: OverlayHitShape,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControllerSkin {
    pub name: String,
    pub image: Option<String>,
    pub buttons: Vec<ControllerSkinButton>,
}

impl ControllerSkin {
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut skin = ControllerSkin::default();
        let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!(
                    "{}:{}: expected key=value",
                    path.display(),
                    line_no + 1
                ));
            };
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            match key {
                "name" => skin.name = value.into(),
                "image" => skin.image = Some(value.into()),
                k if k.starts_with("button.") => skin.buttons.push(parse_button(&k[7..], value)?),
                _ => {}
            }
        }
        Ok(skin)
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Vec<&ControllerSkinButton> {
        self.buttons
            .iter()
            .filter(|button| match button.shape {
                OverlayHitShape::Rect {
                    x: bx,
                    y: by,
                    width,
                    height,
                } => x >= bx && x <= bx + width && y >= by && y <= by + height,
                OverlayHitShape::Circle {
                    x: bx,
                    y: by,
                    radius,
                } => {
                    let dx = x - bx;
                    let dy = y - by;
                    dx * dx + dy * dy <= radius * radius
                }
            })
            .collect()
    }

    pub fn apply_touch(&self, x: f32, y: f32, pressed: bool, input: &mut [[i16; 32]; 8]) {
        for button in self.hit_test(x, y) {
            if let Some(port) = input.get_mut(button.port as usize) {
                if let Some(slot) = port.get_mut(button.retro_id as usize) {
                    *slot = if pressed { 1 } else { 0 };
                }
            }
        }
    }
}

fn parse_button(name: &str, value: &str) -> Result<ControllerSkinButton, String> {
    let fields: Vec<_> = value.split(',').map(str::trim).collect();
    if fields.len() != 6 && fields.len() != 7 {
        return Err(format!(
            "button.{name}: expected retro_id,port,rect,x,y,w,h or retro_id,port,circle,x,y,r"
        ));
    }
    let retro_id = parse_retro_id(fields[0])?;
    let port = fields[1].parse::<u32>().map_err(|e| e.to_string())?;
    let shape = match fields[2] {
        "rect" => {
            if fields.len() != 7 {
                return Err(format!("button.{name}: rect requires x,y,width,height"));
            }
            OverlayHitShape::Rect {
                x: f(fields[3])?,
                y: f(fields[4])?,
                width: f(fields[5])?,
                height: f(fields[6])?,
            }
        }
        "circle" => {
            if fields.len() != 6 {
                return Err(format!("button.{name}: circle requires x,y,radius"));
            }
            OverlayHitShape::Circle {
                x: f(fields[3])?,
                y: f(fields[4])?,
                radius: f(fields[5])?,
            }
        }
        other => return Err(format!("button.{name}: unknown shape {other}")),
    };
    Ok(ControllerSkinButton {
        name: name.into(),
        port,
        retro_id,
        shape,
    })
}

fn f(value: &str) -> Result<f32, String> {
    value.parse::<f32>().map_err(|e| e.to_string())
}

pub fn parse_retro_id(name: &str) -> Result<u32, String> {
    Ok(match name.to_ascii_uppercase().as_str() {
        "B" => DEVICE_ID_JOYPAD_B,
        "Y" => DEVICE_ID_JOYPAD_Y,
        "SELECT" => DEVICE_ID_JOYPAD_SELECT,
        "START" => DEVICE_ID_JOYPAD_START,
        "UP" => DEVICE_ID_JOYPAD_UP,
        "DOWN" => DEVICE_ID_JOYPAD_DOWN,
        "LEFT" => DEVICE_ID_JOYPAD_LEFT,
        "RIGHT" => DEVICE_ID_JOYPAD_RIGHT,
        "A" => DEVICE_ID_JOYPAD_A,
        "X" => DEVICE_ID_JOYPAD_X,
        "L" => DEVICE_ID_JOYPAD_L,
        "R" => DEVICE_ID_JOYPAD_R,
        "L2" => DEVICE_ID_JOYPAD_L2,
        "R2" => DEVICE_ID_JOYPAD_R2,
        "L3" => DEVICE_ID_JOYPAD_L3,
        "R3" => DEVICE_ID_JOYPAD_R3,
        value => value.parse::<u32>().map_err(|e| e.to_string())?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_and_hits_controller_skin() {
        let file =
            std::env::temp_dir().join(format!("retrofront-overlay-{}.cfg", std::process::id()));
        fs::write(&file, "name=Phone\nbutton.a=A,0,circle,0.5,0.5,0.2\nbutton.start=START,0,rect,0.1,0.1,0.2,0.1\n").unwrap();
        let skin = ControllerSkin::load(&file).unwrap();
        assert_eq!(skin.hit_test(0.5, 0.5)[0].retro_id, DEVICE_ID_JOYPAD_A);
        let mut input = [[0i16; 32]; 8];
        skin.apply_touch(0.15, 0.15, true, &mut input);
        assert_eq!(input[0][DEVICE_ID_JOYPAD_START as usize], 1);
        let _ = fs::remove_file(file);
    }
}
