use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::libretro;

#[derive(Debug, Clone, Default)]
pub struct CoreOptionCategory {
    pub _key: String,
    pub _desc: String,
    pub _info: String,
}

#[derive(Debug, Clone, Default)]
pub struct CoreOptionDefinition {
    pub key: String,
    pub desc: String,
    pub info: String,
    pub _category_key: Option<String>,
    pub values: Vec<CoreOptionValue>,
    pub default_value: String,
}

#[derive(Debug, Clone)]
pub struct CoreOptionValue {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Default)]
pub struct CoreOptionsManager {
    _categories: Vec<CoreOptionCategory>,
    definitions: Vec<CoreOptionDefinition>,
    values: HashMap<String, String>,
    c_values_cache: HashMap<String, CString>,
    updated: bool,
    config_path: Option<PathBuf>,
}

impl CoreOptionsManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = Some(path);
        self.load();
    }

    pub fn get_variable_ptr(&mut self, key: &str) -> *const c_char {
        let val = self.values.get(key).or_else(|| {
            self.definitions.iter()
                .find(|d| d.key == key)
                .map(|d| &d.default_value)
        });

        if let Some(val) = val {
            let c_str = CString::new(val.as_str()).unwrap_or_default();
            self.c_values_cache.insert(key.to_string(), c_str);
            self.c_values_cache.get(key).unwrap().as_ptr()
        } else {
            std::ptr::null()
        }
    }

    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.values.get(key).or_else(|| {
            self.definitions.iter()
                .find(|d| d.key == key)
                .map(|d| &d.default_value)
        })
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        self.values.insert(key, value);
        self.updated = true;
        self.save();
    }

    pub fn check_updated(&mut self) -> bool {
        let res = self.updated;
        self.updated = false;
        res
    }

    pub fn set_definitions_v0(&mut self, vars: *const libretro::retro_variable) {
        if vars.is_null() { return; }
        let mut i = 0;
        loop {
            let var = unsafe { &*vars.add(i) };
            if var.key.is_null() || var.value.is_null() { break; }

            let key = unsafe { CStr::from_ptr(var.key) }.to_string_lossy().into_owned();
            let value_str = unsafe { CStr::from_ptr(var.value) }.to_string_lossy();

            let parts: Vec<&str> = value_str.split(';').collect();
            if parts.len() >= 2 {
                let desc = parts[0].trim().to_string();
                let values_part = parts[1].trim();
                let values: Vec<CoreOptionValue> = values_part.split('|').map(|v| {
                    CoreOptionValue { value: v.trim().to_string(), label: v.trim().to_string() }
                }).collect();

                let default_value = values.first().map(|v| v.value.clone()).unwrap_or_default();

                self.add_definition(CoreOptionDefinition {
                    key,
                    desc,
                    info: String::new(),
                    _category_key: None,
                    values,
                    default_value,
                });
            }
            i += 1;
        }
    }

    pub fn set_definitions_v1(&mut self, vars: *const libretro::retro_core_option_definition) {
        if vars.is_null() { return; }
        let mut i = 0;
        loop {
            let var = unsafe { &*vars.add(i) };
            if var.key.is_null() { break; }

            let key = unsafe { CStr::from_ptr(var.key) }.to_string_lossy().into_owned();
            let desc = if var.desc.is_null() { String::new() } else { unsafe { CStr::from_ptr(var.desc) }.to_string_lossy().into_owned() };
            let info = if var.info.is_null() { String::new() } else { unsafe { CStr::from_ptr(var.info) }.to_string_lossy().into_owned() };
            let default_value = if var.default_value.is_null() { String::new() } else { unsafe { CStr::from_ptr(var.default_value) }.to_string_lossy().into_owned() };

            let mut values = Vec::new();
            for j in 0.. {
                let val = &var.values[j];
                if val.value.is_null() { break; }
                values.push(CoreOptionValue {
                    value: unsafe { CStr::from_ptr(val.value) }.to_string_lossy().into_owned(),
                    label: if val.label.is_null() { String::new() } else { unsafe { CStr::from_ptr(val.label) }.to_string_lossy().into_owned() },
                });
            }

            self.add_definition(CoreOptionDefinition {
                key,
                desc,
                info,
                _category_key: None,
                values,
                default_value,
            });
            i += 1;
        }
    }

    pub fn set_definitions_v2(&mut self, definitions: *const libretro::retro_core_option_v2_definition, categories: *const libretro::retro_core_option_v2_category) {
        if !categories.is_null() {
            let mut i = 0;
            self._categories.clear();
            loop {
                let cat = unsafe { &*categories.add(i) };
                if cat.key.is_null() { break; }
                self._categories.push(CoreOptionCategory {
                    _key: unsafe { CStr::from_ptr(cat.key) }.to_string_lossy().into_owned(),
                    _desc: if cat.desc.is_null() { String::new() } else { unsafe { CStr::from_ptr(cat.desc) }.to_string_lossy().into_owned() },
                    _info: if cat.info.is_null() { String::new() } else { unsafe { CStr::from_ptr(cat.info) }.to_string_lossy().into_owned() },
                });
                i += 1;
            }
        }

        if !definitions.is_null() {
            let mut i = 0;
            loop {
                let var = unsafe { &*definitions.add(i) };
                if var.key.is_null() { break; }

                let key = unsafe { CStr::from_ptr(var.key) }.to_string_lossy().into_owned();
                let desc = if var.desc.is_null() { String::new() } else { unsafe { CStr::from_ptr(var.desc) }.to_string_lossy().into_owned() };
                let info = if var.info.is_null() { String::new() } else { unsafe { CStr::from_ptr(var.info) }.to_string_lossy().into_owned() };
                let default_value = if var.default_value.is_null() { String::new() } else { unsafe { CStr::from_ptr(var.default_value) }.to_string_lossy().into_owned() };
                let category_key = if var.category_key.is_null() { None } else { Some(unsafe { CStr::from_ptr(var.category_key) }.to_string_lossy().into_owned()) };

                let mut values = Vec::new();
                for j in 0.. {
                    let val = &var.values[j];
                    if val.value.is_null() { break; }
                    values.push(CoreOptionValue {
                        value: unsafe { CStr::from_ptr(val.value) }.to_string_lossy().into_owned(),
                        label: if val.label.is_null() { String::new() } else { unsafe { CStr::from_ptr(val.label) }.to_string_lossy().into_owned() },
                    });
                }

                self.add_definition(CoreOptionDefinition {
                    key,
                    desc,
                    info,
                    _category_key: category_key,
                    values,
                    default_value,
                });
                i += 1;
            }
        }
    }

    fn add_definition(&mut self, def: CoreOptionDefinition) {
        if let Some(existing) = self.definitions.iter_mut().find(|d| d.key == def.key) {
            *existing = def;
        } else {
            self.definitions.push(def);
        }
    }

    pub fn load(&mut self) {
        let Some(ref path) = self.config_path else { return };
        if !path.exists() { return; }

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') { continue; }
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim().to_string();
                        let value = parts[1].trim().trim_matches('"').to_string();
                        self.values.insert(key, value);
                    }
                }
            }
        }
    }

    pub fn save(&self) {
        let Some(ref path) = self.config_path else { return };
        if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open(path) {
            let mut keys: Vec<&String> = self.values.keys().collect();
            keys.sort();
            for key in keys {
                let value = self.values.get(key).unwrap();
                let _ = writeln!(file, "{} = \"{}\"", key, value);
            }
        }
    }

    pub fn definitions(&self) -> &[CoreOptionDefinition] {
        &self.definitions
    }
}
