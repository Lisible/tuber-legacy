#[derive(Clone)]
pub struct MaterialDescription {
    pub albedo_map: String,
    pub normal_map: Option<String>,
    pub emission_map: Option<String>,
}

impl Default for MaterialDescription {
    fn default() -> Self {
        Self {
            albedo_map: "albedo_texture".to_string(),
            normal_map: None,
            emission_map: None,
        }
    }
}
