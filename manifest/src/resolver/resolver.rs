use crate::Manifest;

pub trait ManifestResolver {
    type Error;
    fn resolve(&self) -> impl Future<Output = Result<Manifest, Self::Error>> + Send + Sync;
}
