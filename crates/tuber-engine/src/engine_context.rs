use tuber_core::asset::Store;
use tuber_core::input::State;

pub struct EngineContext {
    pub asset_store: Store,
    pub input_state: State,
}
