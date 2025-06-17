use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::atomic::AtomicU64,
};

use http::Uri;
use manifest::{Manifest, ManifestResolver};
use tokio::sync::RwLock;

use crate::Error;

#[derive(Debug)]
pub struct HairpinSource {
    location: HairpinSourceLocation,
    manifest: Manifest,
}

impl HairpinSource {
    pub fn new(location: HairpinSourceLocation, manifest: Manifest) -> Self {
        Self { location, manifest }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HairpinSourceLocation {
    Local(PathBuf),
    Remote(Uri),
}
impl HairpinSourceLocation {
    pub fn priority(&self) -> usize {
        match self {
            HairpinSourceLocation::Local(_) => 0,
            HairpinSourceLocation::Remote(_) => 1,
        }
    }
}
impl Ord for HairpinSourceLocation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}
impl PartialOrd for HairpinSourceLocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority().cmp(&other.priority()))
    }
}
impl ManifestResolver for HairpinSourceLocation {
    type Error = Error;

    async fn resolve(&self) -> Result<Manifest, Self::Error> {
        match self {
            HairpinSourceLocation::Local(value) => Ok(value.resolve().await?),
            HairpinSourceLocation::Remote(uri) => todo!(),
        }
    }
}

impl TryFrom<Uri> for HairpinSourceLocation {
    type Error = crate::Error;

    fn try_from(value: Uri) -> Result<Self, Self::Error> {
        if let Some(scheme) = value.scheme_str() {
            match scheme {
                "file" => Ok(HairpinSourceLocation::Local(
                    Path::new(value.path()).to_path_buf(),
                )),
                _ => Err(crate::Error::ProhibitedUri(value.to_string())),
            }
        } else {
            Err(crate::Error::ProhibitedUri(value.to_string()))
        }
    }
}
#[derive(Debug, Default)]
pub struct HairpinDaemon {
    counter: AtomicU64,
    manifests: RwLock<BTreeMap<u64, RwLock<HairpinSource>>>,
}

impl HairpinDaemon {
    pub fn manifests(&self) -> &RwLock<BTreeMap<u64, RwLock<HairpinSource>>> {
        &self.manifests
    }
    pub async fn new_id(&self) -> u64 {
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "cli", derive(clap::Args))]
pub struct HairpinDaemonOptions {
    #[cfg_attr(feature = "cli", arg(long = "disable-mounting"))]
    disable_mounting: bool,
}
