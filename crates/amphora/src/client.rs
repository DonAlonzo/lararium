use crate::Error;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub struct Client {
    registry: Cow<'static, str>,
    cache_dir: PathBuf,
}

impl Client {
    pub fn new(
        registry: impl Into<Cow<'static, str>>,
        cache_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            registry: registry.into(),
            cache_dir: cache_dir.into(),
        }
    }

    pub async fn download(
        &self,
        path: &Path,
        uri: &str,
    ) -> Result<(), Error> {
        Ok(())
    }
}
