use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PackType { Resources, Data }

impl std::fmt::Display for PackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackType::Resources => write!(f, "resources"),
            PackType::Data => write!(f, "data"),
        }
    }
}

impl std::str::FromStr for PackType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "resources" => Ok(PackType::Resources),
            "data" => Ok(PackType::Data),
            _ => Err(format!("Unknown pack type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub format_version: u32,
    pub header: PackHeader,
    #[serde(default)]
    pub modules: Vec<PackModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackHeader {
    pub name: String,
    pub description: String,
    pub uuid: String,
    pub version: [u32; 3],
    #[serde(default)]
    pub min_engine_version: Option<[u32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackModule {
    #[serde(rename = "type")]
    pub module_type: String,
    pub uuid: String,
    pub version: [u32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPack {
    pub manifest: PackManifest,
    pub pack_type: PackType,
    pub uri: String,
    pub source_archive: String,
}
