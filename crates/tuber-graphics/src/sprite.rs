use crate::material::Material;
use crate::texture::TextureRegion;
use crate::MaterialTexture;
use std::time::Instant;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::W;
use tuber_ecs::system::SystemResult;

pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub offset: (f32, f32, i32),
    pub material: Material,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            width: 32.0,
            height: 32.0,
            offset: (0.0, 0.0, 0),
            material: Material {
                albedo_map: MaterialTexture {
                    identifier: "texture".to_string(),
                    region: TextureRegion::new(0.0, 0.0, 32.0, 32.0),
                },
                normal_map: None,
            },
        }
    }
}

pub struct AnimatedSprite {
    pub width: f32,
    pub height: f32,
    pub texture_identifier: String,
    pub animation_state: AnimationState,
}

pub struct AnimationState {
    pub keyframes: Vec<TextureRegion>,
    pub current_keyframe: usize,
    pub start_instant: Instant,
    pub frame_duration: u32,
    pub flip_x: bool,
}

pub fn sprite_animation_step_system(ecs: &mut Ecs) -> SystemResult {
    for (_, (mut animated_sprite,)) in ecs.query::<(W<AnimatedSprite>,)>() {
        let mut animation_state = &mut animated_sprite.animation_state;
        animation_state.current_keyframe = ((animation_state.start_instant.elapsed().as_millis()
            / animation_state.frame_duration as u128)
            % animation_state.keyframes.len() as u128)
            as usize
    }

    Ok(())
}
