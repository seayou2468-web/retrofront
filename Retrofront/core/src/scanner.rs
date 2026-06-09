use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct GameEntry {
    pub path: PathBuf,
    pub label: String,
}

#[derive(Debug, Default)]
pub struct Scanner {
    pub games: Vec<GameEntry>,
}

impl Scanner {
    pub fn new() -> Self {
        Self { games: Vec::new() }
    }

    pub fn scan_directory(&mut self, dir: &Path, extensions: &str) {
        self.games.clear();
        let exts: Vec<&str> = extensions.split('|').collect();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or_default()
                        .to_lowercase();
                    if exts.contains(&ext.as_str()) {
                        let label = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        self.games.push(GameEntry { path, label });
                    }
                }
            }
        }
        self.games.sort_by(|a, b| a.label.cmp(&b.label));
    }
}
