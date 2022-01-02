use crate::engine_context::EngineContext;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::W;
use tuber_ecs::system::{SystemBundle, SystemResult};
use tuber_graphics::sprite::AnimatedSprite;

pub fn default_system_bundle() -> SystemBundle<EngineContext> {
    let mut system_bundle = SystemBundle::<EngineContext>::new();
    system_bundle.add_system(sprite_animation_step_system);
    system_bundle
}

pub fn sprite_animation_step_system(ecs: &mut Ecs, _: &mut EngineContext) -> SystemResult {
    for (_, (mut animated_sprite,)) in ecs.query::<(W<AnimatedSprite>,)>() {
        let mut animation_state = &mut animated_sprite.animation_state;
        animation_state.current_keyframe = ((animation_state.start_instant.elapsed().as_millis()
            / animation_state.frame_duration as u128)
            % animation_state.keyframes.len() as u128)
            as usize
    }

    Ok(())
}
