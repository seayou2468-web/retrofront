use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaylistEntry {
    pub path: PathBuf,
    pub label: String,
    pub core_path: Option<PathBuf>,
    pub core_name: Option<String>,
    pub crc32: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Playlist {
    pub name: String,
    pub entries: Vec<PlaylistEntry>,
}

#[derive(Clone, Debug)]
pub struct PlaylistStore {
    dir: PathBuf,
}

impl PlaylistStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn load(&self, name: &str) -> io::Result<Playlist> {
        let path = self.path_for(name);
        let text = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text).unwrap_or_else(|_| Playlist {
            name: name.into(),
            entries: vec![],
        }))
    }

    pub fn save(&self, playlist: &Playlist) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let text = serde_json::to_string_pretty(playlist).expect("playlist serialize");
        fs::write(self.path_for(&playlist.name), text)
    }

    pub fn list(&self) -> io::Result<Vec<String>> {
        if !self.dir.exists() {
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    names.push(stem.to_owned());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    fn path_for(&self, name: &str) -> PathBuf {
        self.dir.join(Path::new(name).with_extension("json"))
    }
}
