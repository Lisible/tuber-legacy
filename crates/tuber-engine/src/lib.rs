use engine_context::EngineContext;
use state::*;
use tuber_core::asset::AssetStore;
use tuber_core::input::{InputState, Keymap};
use tuber_core::{input, CoreError};
use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;
use tuber_graphics::types::Size2;
use tuber_graphics::{graphics::Graphics, Window};
use tuber_gui::gui::GUI;

pub mod engine_context;
pub mod state;
pub mod system_bundle;

pub struct EngineSettings {
    pub application_title: Option<String>,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            application_title: None,
        }
    }
}

pub struct Engine {
    state_stack: StateStack,
    ecs: Ecs,
    application_title: String,
    context: EngineContext,
    system_bundles: Vec<SystemBundle<EngineContext>>,
}

fn create_ecs() -> Ecs {
    Ecs::new()
}

impl Engine {
    pub fn new(settings: EngineSettings) -> Engine {
        let mut asset_manager = AssetStore::new();
        asset_manager.load_assets_metadata().unwrap();
        asset_manager.register_loaders(Graphics::loaders());

        const KEYMAP_FILE: &'static str = "keymap.json";
        let input_state =
            InputState::new(Keymap::from_file(KEYMAP_FILE).unwrap_or(Keymap::default()));

        let context = EngineContext {
            graphics: Some(Graphics::new()),
            asset_store: asset_manager,
            input_state,
            gui: GUI::new(),
        };

        Self {
            state_stack: StateStack::new(),
            ecs: create_ecs(),
            application_title: settings
                .application_title
                .unwrap_or("tuber Application".into()),
            context,
            system_bundles: vec![],
        }
    }

    pub fn should_exit(&self) -> bool {
        self.state_stack.current_state().is_none()
    }

    pub fn initialize_graphics(&mut self, window: Window, window_size: (u32, u32)) {
        if let Some(graphics) = &mut self.context.graphics {
            graphics.initialize(Window(Box::new(&window)), Size2::from(window_size));
        }
    }

    pub fn application_title(&self) -> &str {
        &self.application_title
    }

    pub fn push_initial_state(&mut self, state: Box<dyn State>) {
        self.state_stack.push_state(
            state,
            &mut self.ecs,
            &mut self.system_bundles,
            &mut self.context,
        )
    }

    pub fn step(&mut self, delta_time: f64) {
        self.state_stack.update_current_state(
            delta_time,
            &mut self.ecs,
            &mut self.system_bundles,
            &mut self.context,
        );
    }

    pub fn handle_input(&mut self, input: input::Input) {
        self.state_stack.handle_input(input, &mut self.context);
    }

    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        self.context
            .graphics
            .as_mut()
            .expect("No graphics")
            .on_window_resized(width, height);
    }

    pub fn render(&mut self) {
        self.state_stack
            .render_current_state(&mut self.ecs, &mut self.context);

        self.context.gui.render(
            self.context.graphics.as_mut().unwrap(),
            &mut self.context.asset_store,
        );

        if let Some(graphics) = self.context.graphics.as_mut() {
            graphics.render_scene(&self.ecs, &mut self.context.asset_store);
        }
    }
}

pub trait TuberRunner {
    fn run(&mut self, engine: Engine) -> Result<()>;
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CoreError(CoreError),
}

impl From<CoreError> for Error {
    fn from(error: CoreError) -> Self {
        Error::CoreError(error)
    }
}
