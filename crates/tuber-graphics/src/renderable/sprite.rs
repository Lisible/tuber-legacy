use crate::material::Material;
use crate::texture::TextureRegion;
use std::time::Instant;

pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub offset: (f32, f32, i32),
    pub texture_region: TextureRegion,
    pub material: Material,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            width: 32.0,
            height: 32.0,
            offset: (0.0, 0.0, 0),
            texture_region: TextureRegion {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            material: Material {
                albedo_map: "texture".to_string(),
                normal_map: None,
            },
        }
    }
}

pub struct AnimatedSprite {
    pub width: f32,
    pub height: f32,
    pub material: Material,
    pub animation_state: AnimationState,
}

pub struct AnimationState {
    pub keyframes: Vec<TextureRegion>,
    pub current_keyframe: usize,
    pub start_instant: Instant,
    pub frame_duration: u32,
    pub flip_x: bool,
}
