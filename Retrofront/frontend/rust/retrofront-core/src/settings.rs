use std::{collections::BTreeMap, fs, io, path::PathBuf};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SettingValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Default, Serialize, Deserialize)]
struct SettingsDocument {
    values: BTreeMap<String, SettingValue>,
}

pub struct SettingsStore {
    path: PathBuf,
    doc: RwLock<SettingsDocument>,
}

impl Clone for SettingsStore {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            doc: RwLock::new(self.doc.read().clone()),
        }
    }
}

impl Clone for SettingsDocument {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
        }
    }
}

impl SettingsStore {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            path: config_dir.join("settings.json"),
            doc: RwLock::default(),
        }
    }

    pub fn load(&self) -> io::Result<()> {
        if self.path.exists() {
            let text = fs::read_to_string(&self.path)?;
            *self.doc.write() = serde_json::from_str(&text).unwrap_or_default();
        }
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(&*self.doc.read()).expect("settings serialize");
        fs::write(&self.path, text)
    }

    pub fn get(&self, key: &str) -> Option<SettingValue> {
        self.doc.read().values.get(key).cloned()
    }

    pub fn set(&self, key: impl Into<String>, value: SettingValue) {
        self.doc.write().values.insert(key.into(), value);
    }
}
