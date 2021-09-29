pub mod input;
pub mod tilemap;
pub mod transform;

pub struct DeltaTime(pub f64);

pub enum CoreError {
    KeymapFileOpenError(std::io::Error),
    KeymapParseError(serde_json::Error),
}
