use std::{
    ops::Not,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use fs_err as fs;

use crate::{Error, Result};

pub struct Dirs {
    inner: ProjectDirs,
}

impl Dirs {
    pub fn init() -> Result<Self> {
        let inner =
            ProjectDirs::from("com", "vrmiguel", "porquinho").ok_or(Error::NoValidHomeDirFound)?;

        let this = Self { inner };

        this.create_dir_if_not_existent(this.config())?;
        this.create_dir_if_not_existent(this.data())?;

        Ok(this)
    }

    fn create_dir_if_not_existent(&self, path: &Path) -> Result<()> {
        if path.exists().not() {
            fs::create_dir_all(path)
                .map_err(|_| Error::CouldNotCreateFolder(PathBuf::from(path)))?;
            println!("info: created folder {:?}", path);
        }

        Ok(())
    }

    pub fn config(&self) -> &Path {
        self.inner.config_dir()
    }

    pub fn data(&self) -> &Path {
        self.inner.data_dir()
    }
}
