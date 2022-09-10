use tuber_core::asset::Store;
use tuber_core::input::State;
use tuber_graphics::Graphics;

pub struct EngineContext {
    pub graphics: Option<Graphics>,
    pub asset_store: Store,
    pub input_state: State,
}
