use tuber_core::asset::AssetStore;
use tuber_core::input::InputState;
use tuber_ecs::system::SystemBundle;
use tuber_graphics::Graphics;

pub struct EngineContext {
    pub graphics: Option<Graphics>,
    pub asset_store: AssetStore,
    pub input_state: InputState,
}
