use std::path::PathBuf;

pub mod asset;
pub mod input;
pub mod transform;

pub type CoreResult<T> = Result<T, CoreError>;
pub struct DeltaTime(pub f64);

#[derive(Debug)]
#[non_exhaustive]
pub enum CoreError {
    KeymapFileOpenError(std::io::Error),
    KeymapParseError(serde_json::Error),
    AssetLoaderNotFound,
    AssetStorageNotFound,
    AssetNotFound,
    AssetDowncastError,
    AssetDescriptionFileNotFound,
    AssetDescriptionFileOpenError(std::io::Error),
    AssetDescriptionFileParseError(serde_json::Error),
    AssetMetadataNotFound,
    CurrentDirInaccessible,
}

pub fn application_directory() -> CoreResult<PathBuf> {
    let manifest_path = std::env::var("CARGO_MANIFEST_DIR");
    if let Ok(manifest_path) = manifest_path {
        return Ok(PathBuf::from(manifest_path));
    }

    let mut path = std::env::current_exe().map_err(|_| CoreError::CurrentDirInaccessible)?;
    path.pop();
    Ok(path)
}
