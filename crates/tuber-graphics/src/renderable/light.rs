use crate::Color;

#[derive(Debug, Clone)]
pub struct PointLight {
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}
