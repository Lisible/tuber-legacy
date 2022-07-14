use std::any::Any;
use std::fs::File;
use std::io::BufReader;

use tuber_core::asset::AssetMetadata;

use crate::WGPUShaderModule;

pub struct Shader {
    inner: WGPUShaderModule,
}

impl Shader {
    pub fn new(inner: WGPUShaderModule) -> Self {
        Self {
            inner
        }
    }
}

pub struct ShaderAsset {
    source: String,
}

pub(crate) fn shader_loader(asset_metadata: &AssetMetadata) -> Box<dyn Any> {
    use image::io::Reader as ImageReader;
    let mut file_path = asset_metadata.asset_path.clone();
    file_path.push(asset_metadata.metadata.get("source_file").unwrap());
    let source = std::fs::read_to_string(file_path).expect(&format!("Failed to read shader {}", asset_metadata.identifier));

    Box::new(ShaderAsset {
        source
    })
}