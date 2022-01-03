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
}
