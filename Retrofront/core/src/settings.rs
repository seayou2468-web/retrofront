use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Settings {
    pub values: HashMap<String, String>,
    pub path: PathBuf,
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, path: &Path) {
        self.path = path.to_path_buf();
        if !path.exists() {
            return;
        }

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
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
        if self.path.as_os_str().is_empty() {
            return;
        }
        if let Ok(mut file) = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
        {
            let mut keys: Vec<&String> = self.values.keys().collect();
            keys.sort();
            for key in keys {
                let value = self.values.get(key).unwrap();
                let _ = writeln!(file, "{} = \"{}\"", key, value);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn libretro_info_path(&self) -> PathBuf {
        self.get("libretro_info_path").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("info"))
    }

    pub fn libretro_directory(&self) -> PathBuf {
        self.get("libretro_directory").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("cores"))
    }

    pub fn content_directory(&self) -> PathBuf {
        self.get("content_directory").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("roms"))
    }

    pub fn savefile_directory(&self) -> PathBuf {
        self.get("savefile_directory").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("saves"))
    }

    pub fn savestate_directory(&self) -> PathBuf {
        self.get("savestate_directory").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("states"))
    }

    pub fn system_directory(&self) -> PathBuf {
        self.get("system_directory").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("system"))
    }
}
