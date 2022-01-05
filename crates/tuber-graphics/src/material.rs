#[derive(Clone)]
pub struct Material {
    pub albedo_map: String,
    pub normal_map: Option<String>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo_map: "albedo_texture".to_string(),
            normal_map: None,
        }
    }
}
