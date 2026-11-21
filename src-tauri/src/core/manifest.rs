use crate::models::{PackManifest, PackType};
use std::io::Read;
use zip::ZipArchive;

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("ZIP error: {0}")] Zip(#[from] zip::result::ZipError),
    #[error("IO error: {0}")] Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")] Json(#[from] serde_json::Error),
    #[error("manifest.json not found")] NotFound,
    #[error("Invalid manifest: {0}")] Invalid(String),
}

pub fn parse_manifest_from_zip<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Result<PackManifest, ManifestError> {
    let mut manifest_content = None;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().ends_with("manifest.json") {
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            manifest_content = Some(buf);
            break;
        }
    }
    let content = manifest_content.ok_or(ManifestError::NotFound)?;
    let manifest: PackManifest = serde_json::from_str(&content)?;
    if manifest.header.uuid.is_empty() {
        return Err(ManifestError::Invalid("UUID is empty".into()));
    }
    Ok(manifest)
}

pub fn determine_pack_type(manifest: &PackManifest) -> Result<PackType, ManifestError> {
    if manifest.modules.is_empty() {
        return Err(ManifestError::Invalid("No modules found".into()));
    }
    for module in &manifest.modules {
        match module.module_type.as_str() {
            "data" => return Ok(PackType::Data),
            "resources" => return Ok(PackType::Resources),
            _ => continue,
        }
    }
    Err(ManifestError::Invalid("Could not determine pack type".into()))
}
