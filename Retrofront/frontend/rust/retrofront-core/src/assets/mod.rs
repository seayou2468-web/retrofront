use std::path::{Path, PathBuf};
#[derive(Clone, Debug)]
pub struct AssetResolver {
    root: PathBuf,
    driver: String,
}
impl AssetResolver {
    pub fn new(root: impl Into<PathBuf>, driver: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            driver: driver.into(),
        }
    }
    pub fn theme_dir(&self) -> PathBuf {
        self.root.join("themes").join(&self.driver)
    }
    pub fn icon(&self, name: &str) -> PathBuf {
        self.root.join("icons").join(format!("{name}.svg"))
    }
    pub fn wallpaper(&self) -> PathBuf {
        self.root
            .join("wallpapers")
            .join(format!("{}.svg", self.driver))
    }
    pub fn placeholder_thumbnail(&self) -> PathBuf {
        self.root.join("thumbnails").join("placeholder.svg")
    }
    pub fn exists_or_placeholder(&self, path: &Path) -> PathBuf {
        if path.exists() {
            path.into()
        } else {
            self.placeholder_thumbnail()
        }
    }
}
