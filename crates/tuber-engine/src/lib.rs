#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

use std::path::PathBuf;

use log::info;

use engine_context::EngineContext;
use state::{State, StateStack};
use tuber_core::asset::Store;
use tuber_core::input::{Keymap, State as InputState};
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
    #[must_use]
    pub fn new(settings: EngineSettings) -> Engine {
        info!("Creating tuber instance");
        let mut asset_manager = Store::default();
        asset_manager.load_assets_metadata().unwrap();

        let input_state = InputState::new(
            Keymap::from_file(&Self::keymap_file_path().unwrap()).unwrap_or_default(),
        );

        let context = EngineContext {
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

    pub fn handle_input(&mut self, input: &input::Input) {
        self.state_stack.handle_input(input, &mut self.context);
    }

    #[allow(clippy::unused_self)]
    pub fn on_window_resized(&mut self, _width: u32, _height: u32) {}

    pub fn render(&mut self) {}

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
