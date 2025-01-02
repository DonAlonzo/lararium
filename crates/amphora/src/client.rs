use crate::Error;
use std::borrow::Cow;
use std::path::Path;

pub struct Client<'a> {
    registry: Cow<'a, str>,
    cache_dir: Cow<'a, Path>,
}

impl<'a> Client<'a> {
    pub fn new(
        registry: impl Into<Cow<'a, str>>,
        cache_dir: impl Into<Cow<'a, Path>>,
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
