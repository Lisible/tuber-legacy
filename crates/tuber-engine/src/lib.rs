use std::path::PathBuf;

use log::info;

use engine_context::EngineContext;
use state::*;
use tuber_core::asset::AssetStore;
use tuber_core::input::{InputState, Keymap};
use tuber_core::{input, CoreError};
use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;

pub mod engine_context;
pub mod state;

#[derive(Default)]
pub struct EngineSettings {
    pub application_title: Option<String>,
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
    pub fn new(settings: EngineSettings) -> Engine {
        info!("Creating tuber instance");
        let mut asset_manager = AssetStore::default();
        asset_manager.load_assets_metadata().unwrap();

        let input_state = InputState::new(
            Keymap::from_file(&Self::keymap_file_path().unwrap()).unwrap_or_default(),
        );

        let context = EngineContext {
            graphics: None,
            asset_store: asset_manager,
            input_state,
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

    pub fn should_exit(&self) -> bool {
        self.state_stack.current_state().is_none()
    }

    pub fn application_title(&self) -> &str {
        &self.application_title
    }

    pub fn push_initial_state(&mut self) {
        self.state_stack.push_initial_state(
            &mut self.ecs,
            &mut self.system_bundles,
            &mut self.context,
        );
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

    pub fn on_window_resized(&mut self, _width: u32, _height: u32) {
        unimplemented!()
    }

    pub fn render(&mut self) {
        self.state_stack
            .render_current_state(&mut self.ecs, &mut self.context);
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
