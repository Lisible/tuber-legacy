use crate::TextureRegion;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationState {
    pub keyframes: Vec<TextureRegion>,
    pub current_keyframe: usize,
    pub start_instant: Instant,
    pub frame_duration: u32,
    pub flip_x: bool,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            keyframes: vec![],
            current_keyframe: 0,
            start_instant: Instant::now(),
            frame_duration: 500,
            flip_x: false,
        }
    }
}

impl AnimationState {
    pub fn update_animation_state(&mut self) {
        self.current_keyframe = ((self.start_instant.elapsed().as_millis()
            / self.frame_duration as u128)
            % self.keyframes.len() as u128) as usize;
    }
}
