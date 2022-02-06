#[derive(Clone)]
pub struct MaterialDescriptor {
    pub albedo_map: String,
    pub normal_map: Option<String>,
    pub emission_map: Option<String>,
}

impl Default for MaterialDescriptor {
    fn default() -> Self {
        Self {
            albedo_map: "albedo_texture".to_string(),
            normal_map: None,
            emission_map: None,
        }
    }
}
