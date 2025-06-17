use std::path::{Path, PathBuf};

use crate::Manifest;

use super::ManifestResolver;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    InvalidManifest(#[from] toml::de::Error),
}
async fn resolve_path(value: impl AsRef<Path>) -> Result<Manifest, Error> {
    let value = value.as_ref();
    let manifest_file = if value.is_dir() {
        value.join(Manifest::NAME)
    } else {
        value.to_path_buf()
    };
    let manifest = toml::from_str(tokio::fs::read_to_string(manifest_file).await?.as_str())?;
    Ok(manifest)
}
impl ManifestResolver for PathBuf {
    type Error = Error;

    async fn resolve(&self) -> Result<Manifest, Self::Error> {
        resolve_path(self).await
    }
}
impl ManifestResolver for &Path {
    type Error = Error;

    async fn resolve(&self) -> Result<Manifest, Self::Error> {
        resolve_path(self).await
    }
}
