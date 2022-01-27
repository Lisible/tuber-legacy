use crate::Color;

#[derive(Debug, Clone)]
pub struct PointLight {
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub radius: f32,
}
