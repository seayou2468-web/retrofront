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
        let wanted = normalize_extensions(extensions);
        self.scan_directory_inner(dir, &wanted);
    }

    fn scan_directory_inner(&mut self, dir: &Path, extensions: &[String]) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if !is_excluded_library_directory(&path) {
                        self.scan_directory_inner(&path, extensions);
                    }
                } else if should_include(&path, extensions)
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

fn normalize_extensions(extensions: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = extensions
        .iter()
        .flat_map(|ext| ext.split('|'))
        .map(|ext| ext.trim().trim_start_matches('.').to_lowercase())
        .filter(|ext| !ext.is_empty() && !blocked_non_rom_extensions().contains(&ext.as_str()))
        .collect();
    for ext in default_rom_extensions() {
        if !normalized.iter().any(|existing| existing == ext) {
            normalized.push((*ext).to_string());
        }
    }
    normalized.sort();
    normalized.dedup();
    normalized
}

fn is_excluded_library_directory(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "cores" | "core" | "info" | "assets" | "system" | "saves" | "states" | "cache" | "logs"
    ) || name.ends_with(".framework")
}

fn should_include(path: &Path, extensions: &[String]) -> bool {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
    {
        return false;
    }
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };
    let ext = ext.to_lowercase();
    if blocked_non_rom_extensions().contains(&ext.as_str()) {
        return false;
    }
    extensions
        .iter()
        .any(|wanted| wanted.eq_ignore_ascii_case(&ext))
}

fn blocked_non_rom_extensions() -> &'static [&'static str] {
    &[
        "txt", "md", "json", "cfg", "conf", "ini", "log", "png", "jpg", "jpeg", "gif", "bmp",
        "webp", "mp3", "flac", "wav", "ogg", "mp4", "m4v", "mov", "avi", "mkv", "srt", "pdf",
        "dylib", "so", "dll", "info", "lpl", "sav", "srm", "state",
    ]
}

fn default_rom_extensions() -> &'static [&'static str] {
    &[
        "nes", "fds", "smc", "sfc", "fig", "gba", "gb", "gbc", "sgb", "n64", "z64", "v64", "nds",
        "sms", "gg", "sg", "md", "gen", "smd", "32x", "pce", "cue", "chd", "iso", "cso", "pbp",
        "bin", "a26", "a52", "a78", "lnx", "ngp", "ngc", "ws", "wsc", "col", "d64", "tap", "prg",
        "zip", "7z",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("retrofront-{name}-{id}"))
    }

    #[test]
    fn scans_default_rom_extensions_even_when_core_filter_is_narrow() {
        let dir = temp_dir("rom-union-filter");
        fs::create_dir_all(&dir).unwrap();
        File::create(dir.join("advance.gba")).unwrap();
        File::create(dir.join("super.sfc")).unwrap();
        File::create(dir.join("notes.txt")).unwrap();

        let mut scanner = Scanner::new();
        scanner.scan_directory(&dir, &["gba".to_string()]);

        let mut labels: Vec<_> = scanner
            .games
            .iter()
            .map(|game| game.label.as_str())
            .collect();
        labels.sort();
        assert_eq!(labels, vec!["advance", "super"]);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_non_rom_files_when_extension_filter_is_empty() {
        let dir = temp_dir("rom-filter");
        fs::create_dir_all(&dir).unwrap();
        File::create(dir.join("game.gba")).unwrap();
        File::create(dir.join("notes.txt")).unwrap();
        File::create(dir.join("cover.png")).unwrap();
        File::create(dir.join("core.dylib")).unwrap();
        fs::create_dir_all(dir.join("Cores")).unwrap();
        File::create(dir.join("Cores/misplaced.gba")).unwrap();

        let mut scanner = Scanner::new();
        scanner.scan_directory(&dir, &[]);

        let labels: Vec<_> = scanner
            .games
            .iter()
            .map(|game| game.label.as_str())
            .collect();
        assert_eq!(labels, vec!["game"]);
        let _ = fs::remove_dir_all(dir);
    }
}
