mod error;

pub use self::error::{Error, Result};

use std::env::current_dir;
use std::fmt::Display;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn new_from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let path = if path.is_absolute() {
            path
        } else {
            current_dir()?.join(path)
        };
        create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub fn load(
        &self,
        path: impl Into<PathBuf>,
    ) -> Result<Vec<u8>> {
        let path = self.path.join(path.into());
        match std::fs::read(&path) {
            Ok(data) => Ok(data),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(Error::NotFound),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(
        &self,
        path: impl Into<PathBuf>,
        data: impl AsRef<[u8]>,
    ) -> Result<()> {
        let path = self.path.join(path.into());
        std::fs::write(path, data)?;
        Ok(())
    }
}

impl FromStr for Store {
    type Err = Error;

    fn from_str(path: &str) -> Result<Self> {
        Self::new_from_path(path)
    }
}

impl Display for Store {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "Store")
    }
}
