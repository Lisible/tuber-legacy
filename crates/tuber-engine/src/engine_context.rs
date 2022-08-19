use tuber_core::asset::Store;
use tuber_core::input::State as InputState;

pub struct EngineContext {
    pub asset_store: Store,
    pub input_state: InputState,
}
