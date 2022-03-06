use std::default::Default;
use tuber_ecs::system::SystemBundle;

use crate::engine_context::EngineContext;

pub fn default_system_bundle() -> SystemBundle<EngineContext> {
    SystemBundle::<EngineContext>::default()
}
