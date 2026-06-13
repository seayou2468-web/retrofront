use std::{
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

include!(concat!(env!("OUT_DIR"), "/reference_dirs.rs"));

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
    pub fn system_dir(&self) -> PathBuf {
        self.root.join("system")
    }
    pub fn cores_dir(&self) -> PathBuf {
        self.root.join("cores")
    }
    pub fn assets_dir(&self) -> PathBuf {
        self.root.join("assets")
    }
    pub fn overlays_dir(&self) -> PathBuf {
        self.root.join("overlays")
    }
    pub fn fonts_dir(&self) -> PathBuf {
        self.assets_dir().join("fonts")
    }
    pub fn roms_dir(&self) -> PathBuf {
        self.root.join("roms")
    }
    pub fn imports_dir(&self) -> PathBuf {
        self.roms_dir().join("imports")
    }
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }
    pub fn screenshots_dir(&self) -> PathBuf {
        self.root.join("screenshots")
    }

    /// Create a RetroArch-compatible directory tree plus Retrofront writable
    /// directories.  `REFERENCE_RETROARCH_DIRS` is generated from
    /// `reference/RetroArch` at build time so the app data tree mirrors the
    /// checked-in reference directory layout.
    pub fn ensure_layout(&self) -> io::Result<()> {
        fs::create_dir_all(&self.root)?;
        for rel in REFERENCE_RETROARCH_DIRS {
            fs::create_dir_all(self.root.join(rel))?;
        }
        for dir in [
            self.config_dir(),
            self.playlists_dir(),
            self.shader_dir(),
            self.saves_dir(),
            self.states_dir(),
            self.system_dir(),
            self.cores_dir(),
            self.assets_dir(),
            self.overlays_dir(),
            self.fonts_dir(),
            self.roms_dir(),
            self.imports_dir(),
            self.logs_dir(),
            self.screenshots_dir(),
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    pub fn read(&self, relative: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        fs::read(self.safe_join(relative.as_ref())?)
    }

    pub fn write(&self, relative: impl AsRef<Path>, bytes: &[u8]) -> io::Result<()> {
        let path = self.safe_join(relative.as_ref())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)
    }

    pub fn copy_into_imports(&self, source: impl AsRef<Path>) -> io::Result<PathBuf> {
        let source = source.as_ref();
        let file_name = source.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "source has no file name")
        })?;
        fs::create_dir_all(self.imports_dir())?;
        let dest = self.imports_dir().join(file_name);
        fs::copy(source, &dest)?;
        Ok(dest)
    }

    pub fn unpack_resources_zip(&self, zip_path: impl AsRef<Path>) -> io::Result<usize> {
        self.unpack_zip_to(zip_path, self.assets_dir())
    }

    pub fn unpack_zip_to(
        &self,
        zip_path: impl AsRef<Path>,
        dest: impl AsRef<Path>,
    ) -> io::Result<usize> {
        let file = fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(zip_err)?;
        let dest = dest.as_ref();
        fs::create_dir_all(dest)?;
        let mut extracted = 0;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(zip_err)?;
            let Some(name) = entry.enclosed_name() else {
                continue;
            };
            let out = dest.join(name);
            if entry.is_dir() {
                fs::create_dir_all(&out)?;
            } else {
                if let Some(parent) = out.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut buf = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut buf)?;
                fs::write(out, buf)?;
                extracted += 1;
            }
        }
        Ok(extracted)
    }

    fn safe_join(&self, relative: &Path) -> io::Result<PathBuf> {
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path must be relative and stay inside app root",
            ));
        }
        Ok(self.root.join(relative))
    }
}

fn zip_err(err: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}
