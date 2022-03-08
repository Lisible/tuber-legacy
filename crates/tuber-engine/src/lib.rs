use std::path::PathBuf;

use engine_context::EngineContext;
use state::*;
use tuber_core::asset::AssetStore;
use tuber_core::input::{InputState, Keymap};
use tuber_core::{input, CoreError};
use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;
use tuber_graphics::GraphicsResult;
use tuber_graphics::{graphics::Graphics, Window};
use tuber_gui::gui::GUI;

pub mod engine_context;
pub mod state;
pub mod system_bundle;

/// The settings for the engine
#[derive(Default)]
pub struct EngineSettings {
    /// The title of the application
    pub application_title: Option<String>,
    /// The initial state of the application
    pub initial_state: Option<Box<dyn State>>,
}

pub struct Engine {
    state_stack: StateStack,
    ecs: Ecs,
    application_title: String,
    context: EngineContext,
    system_bundles: Vec<SystemBundle<EngineContext>>,
}

fn create_ecs() -> Ecs {
    Ecs::default()
}

impl Engine {
    /// Creates an engine instance with the given settings
    pub fn new(settings: EngineSettings) -> Engine {
        let mut asset_manager = AssetStore::default();
        asset_manager.load_assets_metadata().unwrap();

        let input_state = InputState::new(
            Keymap::from_file(&Self::keymap_file_path().unwrap()).unwrap_or_default(),
        );

        let context = EngineContext {
            graphics: Graphics::default(),
            asset_store: asset_manager,
            input_state,
            gui: GUI::default(),
        };

        Self {
            state_stack: StateStack::new(settings.initial_state),
            ecs: create_ecs(),
            application_title: settings
                .application_title
                .unwrap_or_else(|| "tuber Application".into()),
            context,
            system_bundles: vec![],
        }
    }

    pub fn initialize_graphics(&mut self, window: Window, window_size: (u32, u32)) {
        self.context.graphics.initialize(window, window_size);
    }

    /// Pushes the initial state on the state stack
    pub fn push_initial_state(&mut self) {
        self.state_stack.push_initial_state(
            &mut self.ecs,
            &mut self.system_bundles,
            &mut self.context,
        );
    }

    /// Updates the current game state
    pub fn step(&mut self, delta_time: f64) {
        self.state_stack.update_current_state(
            delta_time,
            &mut self.ecs,
            &mut self.system_bundles,
            &mut self.context,
        );
    }

    /// Handles the game's input
    pub fn handle_input(&mut self, input: input::Input) {
        self.state_stack.handle_input(input, &mut self.context);
    }

    /// Called when the window is resized
    pub fn on_window_resized(&mut self, _width: u32, _height: u32) {}

    /// Renders the game
    pub fn render(&mut self) -> GraphicsResult<()> {
        self.state_stack
            .render_current_state(&mut self.ecs, &mut self.context);
        self.context
            .gui
            .render(&mut self.context.graphics, &mut self.context.asset_store);

        self.context.graphics.render_scene()
    }

    /// Returns true if the engine should exit
    pub fn should_exit(&self) -> bool {
        self.state_stack.current_state().is_none()
    }

    pub fn application_title(&self) -> &str {
        &self.application_title
    }

    fn keymap_file_path() -> Result<PathBuf> {
        let mut path = tuber_core::application_directory()?;
        path.push("keymap.json");
        Ok(path)
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
