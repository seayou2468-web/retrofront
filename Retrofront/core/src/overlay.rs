//! RetroArch-compatible input overlay support ported to Rust.
//!
//! The parser and runtime mirror RetroArch's `input_overlay.h` and
//! `tasks/task_overlay.c`: `.cfg` files may define multiple named overlays,
//! descriptor hitboxes, per-descriptor images, next-overlay targets, range/reach
//! modifiers, analog controls and dpad/ABXY eight-way areas. The runtime keeps
//! touch state in normalized video/output coordinates and exposes both libretro
//! joypad state and render descriptors to the host frontend.

use image::RgbaImage;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const OVERLAY_MAX_TOUCH: usize = 16;

const DESC_MOVABLE: u32 = 1 << 0;
const DESC_EXCLUSIVE: u32 = 1 << 1;
const DESC_RANGE_MOD_EXCLUSIVE: u32 = 1 << 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayHitbox {
    Radial,
    Rect,
    None,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayDescType {
    Buttons,
    AnalogLeft,
    AnalogRight,
    DpadArea,
    AbxyArea,
    Keyboard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayVisibility {
    Default,
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Default)]
pub struct OverlayImage {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OverlayDesc {
    pub input: String,
    pub hitbox: OverlayHitbox,
    pub desc_type: OverlayDescType,
    pub next_index: usize,
    pub next_index_name: String,
    pub image_index: Option<usize>,
    pub image: Option<OverlayImage>,
    pub alpha_mod: f32,
    pub range_mod: f32,
    pub analog_saturate_pct: f32,
    pub x: f32,
    pub y: f32,
    pub x_shift: f32,
    pub y_shift: f32,
    pub range_x: f32,
    pub range_y: f32,
    pub mod_x: f32,
    pub mod_y: f32,
    pub mod_w: f32,
    pub mod_h: f32,
    pub reach_up: f32,
    pub reach_down: f32,
    pub reach_left: f32,
    pub reach_right: f32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub flags: u32,
    pub button_mask: HashSet<u32>,
    pub retro_key_idx: u32,
    pub eightway: Option<EightWayConfig>,
}

impl Default for OverlayDesc {
    fn default() -> Self {
        Self {
            input: String::new(),
            hitbox: OverlayHitbox::None,
            desc_type: OverlayDescType::Buttons,
            next_index: 0,
            next_index_name: String::new(),
            image_index: None,
            image: None,
            alpha_mod: 1.0,
            range_mod: 1.0,
            analog_saturate_pct: 1.0,
            x: 0.0,
            y: 0.0,
            x_shift: 0.0,
            y_shift: 0.0,
            range_x: 0.0,
            range_y: 0.0,
            mod_x: 0.0,
            mod_y: 0.0,
            mod_w: 0.0,
            mod_h: 0.0,
            reach_up: 1.0,
            reach_down: 1.0,
            reach_left: 1.0,
            reach_right: 1.0,
            delta_x: 0.0,
            delta_y: 0.0,
            flags: 0,
            button_mask: HashSet::new(),
            retro_key_idx: 0,
            eightway: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EightWayConfig {
    pub up: HashSet<u32>,
    pub right: HashSet<u32>,
    pub down: HashSet<u32>,
    pub left: HashSet<u32>,
    pub up_right: HashSet<u32>,
    pub up_left: HashSet<u32>,
    pub down_right: HashSet<u32>,
    pub down_left: HashSet<u32>,
    pub slope_low: f32,
    pub slope_high: f32,
}

#[derive(Debug, Clone)]
pub struct Overlay {
    pub name: String,
    pub descs: Vec<OverlayDesc>,
    pub image: Option<OverlayImage>,
    pub load_images: Vec<OverlayImage>,
    pub alpha_mod: f32,
    pub range_mod: f32,
    pub normalized: bool,
    pub full_screen: bool,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub aspect_ratio: f32,
    pub viewport: Option<OverlayRect>,
    pub viewport_fill: bool,
    pub block_x_separation: bool,
    pub block_y_separation: bool,
    pub auto_x_separation: bool,
    pub auto_y_separation: bool,
    pub visibility: OverlayVisibility,
}
impl Default for Overlay {
    fn default() -> Self {
        Self {
            name: String::new(),
            descs: Vec::new(),
            image: None,
            load_images: Vec::new(),
            alpha_mod: 1.0,
            range_mod: 1.0,
            normalized: false,
            full_screen: false,
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            center_x: 0.5,
            center_y: 0.5,
            aspect_ratio: 16.0 / 9.0,
            viewport: None,
            viewport_fill: false,
            block_x_separation: false,
            block_y_separation: false,
            auto_x_separation: false,
            auto_y_separation: false,
            visibility: OverlayVisibility::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OverlayRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct OverlayTouch {
    pub active: bool,
    pub x: f32,
    pub y: f32,
}
#[derive(Debug, Clone, Default)]
pub struct OverlayInputState {
    pub buttons: [i16; 16],
    pub analog_left: [i16; 2],
    pub analog_right: [i16; 2],
    pub pointer: Option<(f32, f32)>,
}
#[derive(Debug, Clone)]
pub struct OverlayRenderDesc {
    pub image_path: PathBuf,
    pub image_index: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone)]
pub struct OverlayManager {
    enabled: bool,
    hidden: bool,
    opacity: f32,
    scale_factor: f32,
    active_index: usize,
    overlays: Vec<Overlay>,
    touches: [OverlayTouch; OVERLAY_MAX_TOUCH],
    input_state: OverlayInputState,
    menu_toggle_requested: bool,
    last_path: Option<PathBuf>,
}
impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayManager {
    pub fn new() -> Self {
        Self {
            enabled: false,
            hidden: false,
            opacity: 0.7,
            scale_factor: 1.0,
            active_index: 0,
            overlays: Vec::new(),
            touches: [OverlayTouch::default(); OVERLAY_MAX_TOUCH],
            input_state: OverlayInputState::default(),
            menu_toggle_requested: false,
            last_path: None,
        }
    }
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        let conf = OverlayConfig::load(path)?;
        let overlays_count = conf
            .get_usize("overlays")
            .ok_or("overlay config missing overlays count")?;
        let mut images = ImageCache::default();
        let mut overlays = Vec::with_capacity(overlays_count);
        for idx in 0..overlays_count {
            overlays.push(load_overlay(&conf, path, idx, &mut images)?);
        }
        resolve_targets(&mut overlays)?;
        self.overlays = overlays;
        self.active_index = 0;
        self.last_path = Some(path.to_path_buf());
        self.enabled = true;
        self.recompute_input_state();
        Ok(())
    }
    pub fn unload(&mut self) {
        self.overlays.clear();
        self.active_index = 0;
        self.clear_touches();
        self.last_path = None;
    }
    pub fn enabled(&self) -> bool {
        self.enabled && !self.overlays.is_empty()
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.recompute_input_state();
    }
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor.max(0.01);
    }
    pub fn active_index(&self) -> usize {
        self.active_index
    }
    pub fn active_name(&self) -> Option<&str> {
        self.active_overlay().map(|o| o.name.as_str())
    }
    pub fn overlays(&self) -> &[Overlay] {
        &self.overlays
    }
    pub fn joypad_button(&self, id: u32) -> i16 {
        if id < 16 {
            self.input_state.buttons[id as usize]
        } else {
            0
        }
    }
    pub fn consume_menu_toggle(&mut self) -> bool {
        let requested = self.menu_toggle_requested;
        self.menu_toggle_requested = false;
        requested
    }
    pub fn set_active(&mut self, index: usize) -> Result<(), String> {
        if index >= self.overlays.len() {
            return Err("invalid overlay index".into());
        }
        self.active_index = index;
        self.clear_touches();
        Ok(())
    }

    pub fn set_preferred_orientation(&mut self, portrait: bool) -> Result<(), String> {
        if self.overlays.is_empty() {
            return Err("no overlay loaded".into());
        }
        let preferred = if portrait { "portrait" } else { "landscape" };
        let fallback = self
            .overlays
            .iter()
            .position(|overlay| {
                overlay.name.contains(preferred) && !overlay.name.contains("hidden")
            })
            .or_else(|| {
                self.overlays
                    .iter()
                    .position(|overlay| overlay.name.contains(preferred))
            })
            .unwrap_or(0);
        if self.active_index != fallback {
            self.set_active(fallback)?;
        }
        Ok(())
    }
    pub fn next_overlay(&mut self) {
        if !self.overlays.is_empty() {
            self.active_index = (self.active_index + 1) % self.overlays.len();
            self.clear_touches();
        }
    }
    pub fn set_touch(&mut self, slot: usize, x: f32, y: f32, active: bool) -> Result<(), String> {
        if slot >= OVERLAY_MAX_TOUCH {
            return Err("invalid overlay touch slot".into());
        }
        self.touches[slot] = OverlayTouch {
            active,
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        };
        self.recompute_input_state();
        Ok(())
    }
    pub fn clear_touches(&mut self) {
        self.touches = [OverlayTouch::default(); OVERLAY_MAX_TOUCH];
        self.recompute_input_state();
    }
    pub fn render_descs(&self) -> Vec<OverlayRenderDesc> {
        if !self.enabled() || self.hidden {
            return Vec::new();
        }
        let Some(overlay) = self.active_overlay() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        if let Some(image) = &overlay.image {
            out.push(OverlayRenderDesc {
                image_path: image.path.clone(),
                image_index: 0,
                x: overlay.x,
                y: overlay.y,
                w: overlay.w,
                h: overlay.h,
                alpha: (self.opacity * overlay.alpha_mod).clamp(0.0, 1.0),
            });
        }
        for desc in &overlay.descs {
            if let Some(image) = &desc.image {
                out.push(OverlayRenderDesc {
                    image_path: image.path.clone(),
                    image_index: desc.image_index.unwrap_or(0),
                    x: overlay.x + (desc.x_shift - desc.range_x * self.scale_factor) * overlay.w,
                    y: overlay.y + (desc.y_shift - desc.range_y * self.scale_factor) * overlay.h,
                    w: (desc.range_x * 2.0) * overlay.w * self.scale_factor,
                    h: (desc.range_y * 2.0) * overlay.h * self.scale_factor,
                    alpha: (self.opacity * desc.alpha_mod).clamp(0.0, 1.0),
                });
            }
        }
        out
    }
    pub fn composite_rgba(&self, base: &[u8], width: u32, height: u32) -> Vec<u8> {
        let mut out = base.to_vec();
        if !self.enabled() || self.hidden || width == 0 || height == 0 {
            return out;
        }
        let Some(overlay) = self.active_overlay() else {
            return out;
        };
        if let Some(image) = &overlay.image {
            blend_image(
                &mut out,
                width,
                height,
                image,
                overlay.x,
                overlay.y,
                overlay.w,
                overlay.h,
                (self.opacity * overlay.alpha_mod).clamp(0.0, 1.0),
            );
        }
        for desc in &overlay.descs {
            if let Some(image) = &desc.image {
                let x = overlay.x + (desc.x_shift - desc.range_x * self.scale_factor) * overlay.w;
                let y = overlay.y + (desc.y_shift - desc.range_y * self.scale_factor) * overlay.h;
                let w = desc.range_x * 2.0 * overlay.w * self.scale_factor;
                let h = desc.range_y * 2.0 * overlay.h * self.scale_factor;
                blend_image(
                    &mut out,
                    width,
                    height,
                    image,
                    x,
                    y,
                    w,
                    h,
                    (self.opacity * desc.alpha_mod).clamp(0.0, 1.0),
                );
            }
        }
        out
    }
    fn active_overlay(&self) -> Option<&Overlay> {
        self.overlays.get(self.active_index)
    }
    fn recompute_input_state(&mut self) {
        let mut state = OverlayInputState::default();
        if !self.enabled() {
            self.input_state = state;
            return;
        }
        let Some(overlay) = self.active_overlay() else {
            self.input_state = state;
            return;
        };
        let mut next_target = None;
        let mut menu_toggle_requested = false;
        for touch in self.touches.iter().filter(|t| t.active) {
            state.pointer = Some((touch.x, touch.y));
            let local_x = if overlay.w.abs() > f32::EPSILON {
                (touch.x - overlay.x) / overlay.w
            } else {
                touch.x
            };
            let local_y = if overlay.h.abs() > f32::EPSILON {
                (touch.y - overlay.y) / overlay.h
            } else {
                touch.y
            };
            for desc in &overlay.descs {
                if !desc_hit(desc, local_x, local_y) {
                    continue;
                }
                match desc.desc_type {
                    OverlayDescType::Buttons | OverlayDescType::Keyboard => {
                        for &button in &desc.button_mask {
                            if button < 16 {
                                state.buttons[button as usize] = 1;
                            }
                            if button == 16 {
                                next_target = Some(desc.next_index);
                            }
                            if button == 17 {
                                menu_toggle_requested = true;
                            }
                        }
                    }
                    OverlayDescType::AnalogLeft | OverlayDescType::AnalogRight => {
                        let (x, y) = analog_value(desc, local_x, local_y);
                        let target = if desc.desc_type == OverlayDescType::AnalogLeft {
                            &mut state.analog_left
                        } else {
                            &mut state.analog_right
                        };
                        target[0] = x;
                        target[1] = y;
                    }
                    OverlayDescType::DpadArea | OverlayDescType::AbxyArea => {
                        if let Some(bits) = eightway_buttons(desc, local_x, local_y) {
                            for button in bits {
                                if button < 16 {
                                    state.buttons[button as usize] = 1;
                                }
                            }
                        }
                    }
                }
                if desc.flags & DESC_EXCLUSIVE != 0 {
                    break;
                }
            }
        }
        self.input_state = state;
        if menu_toggle_requested {
            self.menu_toggle_requested = true;
        }
        if let Some(index) = next_target.filter(|i| *i < self.overlays.len()) {
            self.active_index = index;
            self.clear_touches();
        }
    }
}

#[derive(Default)]
struct ImageCache {
    images: HashMap<PathBuf, OverlayImage>,
}
impl ImageCache {
    fn load(&mut self, base: &Path, rel: &str) -> Result<OverlayImage, String> {
        let path = resolve_overlay_path(base, rel);
        if let Some(img) = self.images.get(&path) {
            return Ok(img.clone());
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let image = if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "tga") {
            match image::open(&path) {
                Ok(decoded) => {
                    let rgba: RgbaImage = decoded.to_rgba8();
                    OverlayImage {
                        path: path.clone(),
                        width: rgba.width(),
                        height: rgba.height(),
                        rgba: rgba.into_raw(),
                    }
                }
                Err(_) => OverlayImage {
                    path: path.clone(),
                    width: 0,
                    height: 0,
                    rgba: Vec::new(),
                },
            }
        } else {
            OverlayImage {
                path: path.clone(),
                width: 0,
                height: 0,
                rgba: Vec::new(),
            }
        };
        self.images.insert(path, image.clone());
        Ok(image)
    }
}

#[derive(Debug)]
struct OverlayConfig {
    values: BTreeMap<String, String>,
}
impl OverlayConfig {
    fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|e| format!("failed to read overlay config {}: {e}", path.display()))?;
        let mut values = BTreeMap::new();
        for raw_line in text.lines() {
            let line = raw_line
                .split('#')
                .next()
                .unwrap_or("")
                .split("//")
                .next()
                .unwrap_or("")
                .trim();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            values.insert(
                key.trim().to_string(),
                strip_quotes(value.trim()).to_string(),
            );
        }
        Ok(Self { values })
    }
    fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
    fn get_usize(&self, key: &str) -> Option<usize> {
        self.get(key)?.parse().ok()
    }
    fn get_f32(&self, key: &str) -> Option<f32> {
        self.get(key)?.parse().ok()
    }
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "true" | "1" | "yes" | "on"))
    }
}

fn load_overlay(
    conf: &OverlayConfig,
    cfg_path: &Path,
    idx: usize,
    images: &mut ImageCache,
) -> Result<Overlay, String> {
    let mut overlay = Overlay::default();
    let prefix = format!("overlay{idx}");
    let descs = conf
        .get_usize(&format!("{prefix}_descs"))
        .ok_or_else(|| format!("missing {prefix}_descs"))?;
    overlay.name = conf
        .get(&format!("{prefix}_name"))
        .unwrap_or("")
        .to_string();
    overlay.alpha_mod = conf.get_f32(&format!("{prefix}_alpha_mod")).unwrap_or(1.0);
    overlay.range_mod = conf.get_f32(&format!("{prefix}_range_mod")).unwrap_or(1.0);
    overlay.normalized = conf
        .get_bool(&format!("{prefix}_normalized"))
        .unwrap_or(false);
    overlay.full_screen = conf
        .get_bool(&format!("{prefix}_full_screen"))
        .unwrap_or(false);
    if let Some(path) = conf.get(&format!("{prefix}_overlay")) {
        let image = images.load(cfg_path, path)?;
        overlay.load_images.push(image.clone());
        overlay.image = Some(image);
    }
    overlay.aspect_ratio = conf
        .get_f32(&format!("{prefix}_aspect_ratio"))
        .unwrap_or_else(|| {
            if overlay.name.contains("portrait") {
                9.0 / 16.0
            } else {
                16.0 / 9.0
            }
        });
    if let Some(rect) = conf.get(&format!("{prefix}_rect")).and_then(parse_rect) {
        overlay.x = rect.x;
        overlay.y = rect.y;
        overlay.w = rect.w;
        overlay.h = rect.h;
    }
    overlay.center_x = overlay.x + overlay.w * 0.5;
    overlay.center_y = overlay.y + overlay.h * 0.5;
    overlay.viewport = conf.get(&format!("{prefix}_viewport")).and_then(parse_rect);
    overlay.viewport_fill = conf
        .get_bool(&format!("{prefix}_viewport_fill"))
        .unwrap_or(false);
    overlay.block_x_separation = conf
        .get_bool(&format!("{prefix}_block_x_separation"))
        .unwrap_or(false);
    overlay.block_y_separation = conf
        .get_bool(&format!("{prefix}_block_y_separation"))
        .unwrap_or(false);
    overlay.auto_x_separation = conf
        .get_bool(&format!("{prefix}_auto_x_separation"))
        .unwrap_or_else(|| !overlay.block_x_separation && overlay.image.is_none());
    overlay.auto_y_separation = conf
        .get_bool(&format!("{prefix}_auto_y_separation"))
        .unwrap_or(false);
    let base_w = overlay.image.as_ref().map(|i| i.width).unwrap_or(0);
    let base_h = overlay.image.as_ref().map(|i| i.height).unwrap_or(0);
    for desc_idx in 0..descs {
        let mut desc = load_desc(
            conf,
            cfg_path,
            &prefix,
            desc_idx,
            base_w,
            base_h,
            overlay.normalized,
            overlay.alpha_mod,
            overlay.range_mod,
            images,
        )?;
        if let Some(image) = desc.image.clone() {
            desc.image_index = Some(overlay.load_images.len());
            overlay.load_images.push(image);
        }
        overlay.descs.push(desc);
    }
    Ok(overlay)
}

fn load_desc(
    conf: &OverlayConfig,
    cfg_path: &Path,
    overlay_prefix: &str,
    desc_idx: usize,
    width: u32,
    height: u32,
    default_normalized: bool,
    alpha_mod: f32,
    range_mod: f32,
    images: &mut ImageCache,
) -> Result<OverlayDesc, String> {
    let key = format!("{overlay_prefix}_desc{desc_idx}");
    let normalized = conf
        .get_bool(&format!("{key}_normalized"))
        .unwrap_or(default_normalized);
    let by_pixel = !normalized;
    if by_pixel && (width == 0 || height == 0) {
        return Err(format!(
            "{key} uses pixel coordinates but has no base overlay image"
        ));
    }
    let value = conf.get(&key).ok_or_else(|| format!("missing {key}"))?;
    let elems = split_list(value);
    if elems.len() < 6 {
        return Err(format!("{key} requires at least 6 tokens"));
    }
    let mut desc = OverlayDesc {
        input: elems[0].clone(),
        ..OverlayDesc::default()
    };
    let width_mod = if by_pixel { 1.0 / width as f32 } else { 1.0 };
    let height_mod = if by_pixel { 1.0 / height as f32 } else { 1.0 };
    desc.x = elems[1].parse::<f32>().unwrap_or(0.0) * width_mod;
    desc.y = elems[2].parse::<f32>().unwrap_or(0.0) * height_mod;
    desc.x_shift = desc.x;
    desc.y_shift = desc.y;
    desc.hitbox = match elems[3].as_str() {
        "radial" => OverlayHitbox::Radial,
        "rect" => OverlayHitbox::Rect,
        _ => OverlayHitbox::None,
    };
    desc.range_x = elems[4].parse::<f32>().unwrap_or(0.0) * width_mod;
    desc.range_y = elems[5].parse::<f32>().unwrap_or(0.0) * height_mod;
    desc.desc_type = parse_desc_type(&elems[0], &mut desc);
    if desc.desc_type == OverlayDescType::Buttons {
        for part in elems[0].split('|') {
            if part == "nul" {
                continue;
            }
            let clean = part.strip_suffix("_enable").unwrap_or(part);
            if let Some(id) = bind_id(clean) {
                desc.button_mask.insert(id);
            }
        }
        if desc.button_mask.contains(&16) {
            desc.next_index_name = conf
                .get(&format!("{key}_next_target"))
                .unwrap_or("")
                .to_string();
        }
    }
    if matches!(
        desc.desc_type,
        OverlayDescType::AnalogLeft | OverlayDescType::AnalogRight
    ) {
        if desc.hitbox != OverlayHitbox::Radial {
            return Err(format!("{key} analog hitbox must be radial"));
        }
        desc.analog_saturate_pct = conf.get_f32(&format!("{key}_saturate_pct")).unwrap_or(1.0);
    }
    if matches!(
        desc.desc_type,
        OverlayDescType::DpadArea | OverlayDescType::AbxyArea
    ) {
        desc.eightway = Some(eightway_config(conf, &key, desc.desc_type));
    }
    desc.reach_right = conf.get_f32(&format!("{key}_reach_x")).unwrap_or(1.0);
    desc.reach_left = desc.reach_right;
    desc.reach_up = conf.get_f32(&format!("{key}_reach_y")).unwrap_or(1.0);
    desc.reach_down = desc.reach_up;
    if let Some(v) = conf.get_f32(&format!("{key}_reach_up")) {
        desc.reach_up = v;
    }
    if let Some(v) = conf.get_f32(&format!("{key}_reach_down")) {
        desc.reach_down = v;
    }
    if let Some(v) = conf.get_f32(&format!("{key}_reach_left")) {
        desc.reach_left = v;
    }
    if let Some(v) = conf.get_f32(&format!("{key}_reach_right")) {
        desc.reach_right = v;
    }
    desc.alpha_mod = conf
        .get_f32(&format!("{key}_alpha_mod"))
        .unwrap_or(alpha_mod);
    desc.range_mod = conf
        .get_f32(&format!("{key}_range_mod"))
        .unwrap_or(range_mod);
    if conf.get_bool(&format!("{key}_movable")).unwrap_or(false) {
        desc.flags |= DESC_MOVABLE;
    }
    if conf.get_bool(&format!("{key}_exclusive")).unwrap_or(false) {
        desc.flags |= DESC_EXCLUSIVE;
    }
    if conf
        .get_bool(&format!("{key}_range_mod_exclusive"))
        .unwrap_or(false)
    {
        desc.flags |= DESC_RANGE_MOD_EXCLUSIVE;
    }
    if (desc.reach_left == 0.0 && desc.reach_right == 0.0)
        || (desc.reach_up == 0.0 && desc.reach_down == 0.0)
    {
        desc.hitbox = OverlayHitbox::None;
    }
    desc.mod_x = desc.x - desc.range_x;
    desc.mod_y = desc.y - desc.range_y;
    desc.mod_w = desc.range_x * 2.0;
    desc.mod_h = desc.range_y * 2.0;
    if let Some(path) = conf.get(&format!("{key}_overlay")) {
        desc.image = Some(images.load(cfg_path, path)?);
    }
    Ok(desc)
}

fn resolve_targets(overlays: &mut [Overlay]) -> Result<(), String> {
    let names: HashMap<String, usize> = overlays
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.name.is_empty())
        .map(|(i, o)| (o.name.clone(), i))
        .collect();
    let len = overlays.len();
    for (idx, overlay) in overlays.iter_mut().enumerate() {
        for desc in &mut overlay.descs {
            let mut target = (idx + 1) % len;
            if !desc.next_index_name.is_empty() {
                target = *names
                    .get(&desc.next_index_name)
                    .ok_or_else(|| format!("unknown overlay target {}", desc.next_index_name))?;
            }
            desc.next_index = target;
        }
    }
    Ok(())
}
fn parse_desc_type(name: &str, desc: &mut OverlayDesc) -> OverlayDescType {
    if name.starts_with("analog_left") {
        OverlayDescType::AnalogLeft
    } else if name.starts_with("analog_right") {
        OverlayDescType::AnalogRight
    } else if name.starts_with("dpad_area") {
        OverlayDescType::DpadArea
    } else if name.starts_with("abxy_area") {
        OverlayDescType::AbxyArea
    } else if let Some(key) = name.strip_prefix("retrok_") {
        desc.retro_key_idx = retro_key_id(key);
        OverlayDescType::Keyboard
    } else {
        OverlayDescType::Buttons
    }
}
fn eightway_config(conf: &OverlayConfig, key: &str, ty: OverlayDescType) -> EightWayConfig {
    let mut up = HashSet::new();
    let mut down = HashSet::new();
    let mut left = HashSet::new();
    let mut right = HashSet::new();
    match ty {
        OverlayDescType::AbxyArea => {
            up.insert(9);
            down.insert(0);
            left.insert(1);
            right.insert(8);
        }
        _ => {
            up.insert(4);
            down.insert(5);
            left.insert(6);
            right.insert(7);
        }
    }
    override_bits(conf, &format!("{key}_up"), &mut up);
    override_bits(conf, &format!("{key}_down"), &mut down);
    override_bits(conf, &format!("{key}_left"), &mut left);
    override_bits(conf, &format!("{key}_right"), &mut right);
    let mut up_right = up.clone();
    up_right.extend(right.iter().copied());
    let mut up_left = up.clone();
    up_left.extend(left.iter().copied());
    let mut down_right = down.clone();
    down_right.extend(right.iter().copied());
    let mut down_left = down.clone();
    down_left.extend(left.iter().copied());
    EightWayConfig {
        up,
        right,
        down,
        left,
        up_right,
        up_left,
        down_right,
        down_left,
        slope_low: 0.41421356,
        slope_high: 2.4142137,
    }
}
fn override_bits(conf: &OverlayConfig, key: &str, bits: &mut HashSet<u32>) {
    if let Some(value) = conf.get(key) {
        bits.clear();
        for part in value.split('|') {
            if let Some(id) = bind_id(part) {
                bits.insert(id);
            }
        }
    }
}
fn desc_hit(desc: &OverlayDesc, x: f32, y: f32) -> bool {
    if desc.hitbox == OverlayHitbox::None {
        return false;
    }
    let dx = x - desc.x_shift;
    let dy = y - desc.y_shift;
    let left = desc.range_x * desc.reach_left * desc.range_mod;
    let right = desc.range_x * desc.reach_right * desc.range_mod;
    let up = desc.range_y * desc.reach_up * desc.range_mod;
    let down = desc.range_y * desc.reach_down * desc.range_mod;
    if dx < -left || dx > right || dy < -up || dy > down {
        return false;
    }
    match desc.hitbox {
        OverlayHitbox::Rect => true,
        OverlayHitbox::Radial => {
            let rx = if dx < 0.0 { left } else { right }.max(f32::EPSILON);
            let ry = if dy < 0.0 { up } else { down }.max(f32::EPSILON);
            (dx / rx).powi(2) + (dy / ry).powi(2) <= 1.0
        }
        OverlayHitbox::None => false,
    }
}
fn analog_value(desc: &OverlayDesc, x: f32, y: f32) -> (i16, i16) {
    let mut nx = (x - desc.x_shift) / (desc.range_x * desc.analog_saturate_pct).max(f32::EPSILON);
    let mut ny = (y - desc.y_shift) / (desc.range_y * desc.analog_saturate_pct).max(f32::EPSILON);
    let mag = (nx * nx + ny * ny).sqrt();
    if mag > 1.0 {
        nx /= mag;
        ny /= mag;
    }
    ((nx * 32767.0) as i16, (ny * 32767.0) as i16)
}
fn eightway_buttons(desc: &OverlayDesc, x: f32, y: f32) -> Option<HashSet<u32>> {
    let cfg = desc.eightway.as_ref()?;
    let dx = x - desc.x_shift;
    let dy = y - desc.y_shift;
    if dx.abs() < f32::EPSILON && dy.abs() < f32::EPSILON {
        return None;
    }
    let slope = (dy.abs() / dx.abs().max(f32::EPSILON)).abs();
    let vertical = if dy < 0.0 { &cfg.up } else { &cfg.down };
    let horizontal = if dx < 0.0 { &cfg.left } else { &cfg.right };
    if slope > cfg.slope_high {
        return Some(vertical.clone());
    }
    if slope < cfg.slope_low {
        return Some(horizontal.clone());
    }
    let mut out = vertical.clone();
    out.extend(horizontal.iter().copied());
    Some(out)
}
fn blend_image(
    out: &mut [u8],
    width: u32,
    height: u32,
    image: &OverlayImage,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    alpha: f32,
) {
    if image.rgba.is_empty() || image.width == 0 || image.height == 0 || w <= 0.0 || h <= 0.0 {
        return;
    }
    let dst_x0 = (x * width as f32).round() as i32;
    let dst_y0 = (y * height as f32).round() as i32;
    let dst_w = (w * width as f32).round().max(1.0) as i32;
    let dst_h = (h * height as f32).round().max(1.0) as i32;
    for dy in 0..dst_h {
        let oy = dst_y0 + dy;
        if oy < 0 || oy >= height as i32 {
            continue;
        }
        let sy = (dy as f32 / dst_h as f32 * image.height as f32) as u32;
        for dx in 0..dst_w {
            let ox = dst_x0 + dx;
            if ox < 0 || ox >= width as i32 {
                continue;
            }
            let sx = (dx as f32 / dst_w as f32 * image.width as f32) as u32;
            let sidx =
                ((sy.min(image.height - 1) * image.width + sx.min(image.width - 1)) * 4) as usize;
            let didx = ((oy as u32 * width + ox as u32) * 4) as usize;
            let a = (image.rgba[sidx + 3] as f32 / 255.0 * alpha).clamp(0.0, 1.0);
            for c in 0..3 {
                out[didx + c] =
                    (image.rgba[sidx + c] as f32 * a + out[didx + c] as f32 * (1.0 - a)) as u8;
            }
            out[didx + 3] = 255;
        }
    }
}
fn resolve_overlay_path(base_cfg: &Path, rel: &str) -> PathBuf {
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        rel_path.to_path_buf()
    } else {
        base_cfg
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(rel_path)
    }
}
fn split_list(value: &str) -> Vec<String> {
    value
        .split(|c| c == ',' || c == ' ')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}
fn parse_rect(value: &str) -> Option<OverlayRect> {
    let v = split_list(value);
    if v.len() < 4 {
        return None;
    }
    Some(OverlayRect {
        x: v[0].parse().ok()?,
        y: v[1].parse().ok()?,
        w: v[2].parse().ok()?,
        h: v[3].parse().ok()?,
    })
}
fn strip_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value)
}
fn retro_key_id(key: &str) -> u32 {
    key.bytes()
        .fold(0u32, |a, b| a.wrapping_mul(33).wrapping_add(b as u32))
}
fn bind_id(name: &str) -> Option<u32> {
    match name.to_ascii_lowercase().as_str() {
        "b" => Some(0),
        "y" => Some(1),
        "select" => Some(2),
        "start" => Some(3),
        "up" => Some(4),
        "down" => Some(5),
        "left" => Some(6),
        "right" => Some(7),
        "a" => Some(8),
        "x" => Some(9),
        "l" | "l1" => Some(10),
        "r" | "r1" => Some(11),
        "l2" => Some(12),
        "r2" => Some(13),
        "l3" => Some(14),
        "r3" => Some(15),
        "overlay_next" => Some(16),
        "menu_toggle" | "menu" => Some(17),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn parses_retroarch_overlay_and_maps_touch() {
        let dir = tempfile_dir();
        let cfg = dir.join("test.cfg");
        fs::File::create(&cfg)
            .unwrap()
            .write_all(
                br#"
overlays = 1
overlay0_name = "landscape"
overlay0_descs = 2
overlay0_normalized = true
overlay0_desc0 = "a,0.8,0.5,radial,0.1,0.1"
overlay0_desc1 = "dpad_area,0.2,0.5,rect,0.15,0.15"
"#,
            )
            .unwrap();
        let mut manager = OverlayManager::new();
        manager.load(&cfg).unwrap();
        manager.set_touch(0, 0.8, 0.5, true).unwrap();
        assert_eq!(manager.joypad_button(8), 1);
        manager.set_touch(0, 0.2, 0.35, true).unwrap();
        assert_eq!(manager.joypad_button(4), 1);
    }

    #[test]
    fn selects_orientation_named_overlay() {
        let dir = tempfile_dir();
        let cfg = dir.join("orientation.cfg");
        fs::File::create(&cfg)
            .unwrap()
            .write_all(
                br#"
overlays = 2
overlay0_name = "landscape"
overlay0_descs = 1
overlay0_normalized = true
overlay0_desc0 = "a,0.8,0.5,radial,0.1,0.1"
overlay1_name = "portrait"
overlay1_descs = 1
overlay1_normalized = true
overlay1_desc0 = "b,0.2,0.5,radial,0.1,0.1"
"#,
            )
            .unwrap();
        let mut manager = OverlayManager::new();
        manager.load(&cfg).unwrap();
        manager.set_preferred_orientation(true).unwrap();
        assert_eq!(manager.active_name(), Some("portrait"));
        manager.set_preferred_orientation(false).unwrap();
        assert_eq!(manager.active_name(), Some("landscape"));
    }
    fn tempfile_dir() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("retrofront-overlay-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }
}
