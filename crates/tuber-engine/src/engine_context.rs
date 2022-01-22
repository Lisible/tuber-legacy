use tuber_core::asset::AssetStore;
use tuber_core::input::InputState;
use tuber_graphics::graphics::Graphics;
use tuber_gui::gui::GUI;

pub struct EngineContext {
    pub graphics: Option<Graphics>,
    pub asset_store: AssetStore,
    pub input_state: InputState,
    pub gui: GUI,
}
