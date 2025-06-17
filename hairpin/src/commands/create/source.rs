use std::path::PathBuf;

use clap::Args;
use manifest::Manifest;
use toml::{Value, map::Map};
use uuid::Uuid;

use crate::{Resolver, commands::parse_property};

#[derive(Debug, Args)]
pub struct CreateSourceArgs {
    #[arg(last = true)]
    path: PathBuf,
    #[arg(short = 'n', long = "name")]
    name: Option<String>,
    #[arg(short = 'l', long = "label")]
    labels: Vec<String>,
    #[arg(short = 'p', long = "property",value_parser = parse_property::<String,toml::Value>)]
    properties: Vec<(String, Value)>,
}
impl Resolver for CreateSourceArgs {
    type Context = ();

    type Error = Error;

    fn resolve(self, _: Self::Context) -> Result<(), Self::Error> {
        if self.path.is_dir() {
            if self.path.read_dir()?.into_iter().count() != 0 {
                let mut root = Manifest::builder();
                root.set_id(Uuid::new_v4().to_string());
                if let Some(name) = self.name {
                    root.set_name(name);
                }
                for label in self.labels {
                    root.with_label(label);
                }
                root.set_properties(self.properties.into_iter().collect::<Map<_, _>>());
                root.set_version(env!("CARGO_PKG_VERSION").to_string());
                let root = root.build();
                std::fs::write(
                    self.path.join(Manifest::NAME),
                    toml::to_string_pretty(&root)?,
                )?;
                Ok(())
            } else {
                Err(Error::SourceNotEmpty)
            }
        } else {
            Err(Error::SourceNotDir)
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("must be a directory")]
    SourceNotDir,
    #[error("must be empty")]
    SourceNotEmpty,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Serialization(#[from] toml::ser::Error),
}
