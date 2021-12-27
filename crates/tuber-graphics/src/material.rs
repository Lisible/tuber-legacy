use crate::TextureRegion;

pub struct Material {
    pub albedo_map: MaterialTexture,
    pub normal_map: Option<MaterialTexture>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo_map: MaterialTexture {
                identifier: "albedo_texture".to_string(),
                region: TextureRegion::new(0.0, 0.0, 1.0, 1.0),
            },
            normal_map: None,
        }
    }
}

pub struct MaterialTexture {
    pub identifier: String,
    pub region: TextureRegion,
}
