use crate::animation::AnimationState;
use crate::material::MaterialDescriptor;
use crate::texture::TextureRegion;

pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub texture_region: TextureRegion,
    pub material: MaterialDescriptor,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            width: 32.0,
            height: 32.0,
            texture_region: TextureRegion {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            material: MaterialDescriptor {
                albedo_map: "texture".to_string(),
                normal_map: None,
                emission_map: None,
            },
        }
    }
}

pub struct AnimatedSprite {
    pub width: f32,
    pub height: f32,
    pub material: MaterialDescriptor,
    pub animation_state: AnimationState,
}
