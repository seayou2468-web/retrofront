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

    pub fn clear(&mut self) {
        self.games.clear();
    }

    pub fn scan_directory(&mut self, dir: &Path, extensions: &[String]) {
        let wanted: Vec<String> = extensions
            .iter()
            .map(|ext| ext.trim_start_matches('.').to_lowercase())
            .filter(|ext| !ext.is_empty())
            .collect();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_directory(&path, &wanted);
                } else if should_include(&path, &wanted)
                    && !self.games.iter().any(|g| g.path == path)
                {
                    self.games.push(GameEntry {
                        label: path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        path,
                        core_path: None,
                    });
                }
            }
        }
    }
}

fn should_include(path: &Path, extensions: &[String]) -> bool {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
    {
        return false;
    }
    if extensions.is_empty() {
        return path.extension().is_some();
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            extensions
                .iter()
                .any(|wanted| wanted.eq_ignore_ascii_case(ext))
        })
        .unwrap_or(false)
}
