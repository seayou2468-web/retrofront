use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct HostFilesystem {
    root: PathBuf,
}

impl HostFilesystem {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn config_dir(&self) -> PathBuf {
        self.root.join("config")
    }
    pub fn playlists_dir(&self) -> PathBuf {
        self.root.join("playlists")
    }
    pub fn shader_dir(&self) -> PathBuf {
        self.root.join("shaders")
    }
    pub fn saves_dir(&self) -> PathBuf {
        self.root.join("saves")
    }
    pub fn states_dir(&self) -> PathBuf {
        self.root.join("states")
    }

    pub fn ensure_layout(&self) -> io::Result<()> {
        for dir in [
            self.config_dir(),
            self.playlists_dir(),
            self.shader_dir(),
            self.saves_dir(),
            self.states_dir(),
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    pub fn read(&self, relative: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        fs::read(self.root.join(relative))
    }

    pub fn write(&self, relative: impl AsRef<Path>, bytes: &[u8]) -> io::Result<()> {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)
    }
}
