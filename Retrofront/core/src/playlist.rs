use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistEntry {
    pub path: PathBuf,
    pub label: String,
    pub core_path: PathBuf,
    pub core_name: String,
    pub crc32: String,
    pub db_name: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Playlist {
    pub items: Vec<PlaylistEntry>,
}

impl Playlist {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }
}
