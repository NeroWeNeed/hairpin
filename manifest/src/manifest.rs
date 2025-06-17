use std::path::{Path, PathBuf};

use builder::Builder;
use serde::{Deserialize, Serialize, Serializer};
use toml::{Value, map::Map};
#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct Manifest {
    id: String,
    name: String,
    version: String,
    items: Vec<Item>,
    properties: Map<String, Value>,
    #[builder(setter_name = "label")]
    labels: Vec<String>,
}
impl Manifest {
    pub const NAME: &'static str = "Hairpin.toml";
}
#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct Item {
    id: String,
    name: String,
    value: ValueAccessor,
    encryption: ItemEncryption,
    properties: Map<String, Value>,
    labels: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ItemEncryption {
    #[default]
    None,
    PlainText,
}
#[derive(Clone, Debug, Default)]
pub enum ValueAccessor {
    #[default]
    None,
    Path(PathBuf),
    Remote,
}
impl Serialize for ValueAccessor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let ValueAccessor::Path(value) = self {
            if let Some(value) = value.to_str() {
                return serializer.serialize_str(value);
            }
        }
        serializer.serialize_unit()
    }
}
impl<'de> Deserialize<'de> for ValueAccessor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .map(|value| ValueAccessor::Path(Path::new(value.as_str()).to_path_buf()))
    }
}
