#[derive(Clone)]
pub struct Material {
    pub albedo_map: String,
    pub normal_map: Option<String>,
}

impl Material {
    pub fn new(albedo_map: &str, normal_map: Option<&str>) -> Self {
        Self {
            albedo_map: albedo_map.into(),
            normal_map: normal_map.map(|v| v.to_string()),
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo_map: "albedo_texture".to_string(),
            normal_map: None,
        }
    }
}
