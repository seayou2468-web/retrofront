use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GameEntry {
    pub path: PathBuf,
    pub label: String,
    pub core_path: Option<PathBuf>,
}

pub struct Scanner {
    pub games: Vec<GameEntry>,
}

impl Scanner {
    pub fn new() -> Self {
        Self { games: Vec::new() }
    }

    pub fn scan_directory(&mut self, dir: &Path, extensions: &[String]) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_directory(&path, extensions);
                } else if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                        self.games.push(GameEntry {
                            label: path.file_stem().unwrap().to_string_lossy().into_owned(),
                            path,
                            core_path: None,
                        });
                    }
                }
            }
        }
    }
}
