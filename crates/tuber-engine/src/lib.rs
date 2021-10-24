use state::*;
use tuber_core::asset::AssetStore;
use tuber_core::input::{InputState, Keymap};
use tuber_core::{input, CoreError};
use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;
use tuber_graphics::{Graphics, Window};

pub mod state;

pub struct EngineSettings {
    pub graphics: Option<Graphics>,
    pub application_title: Option<String>,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            graphics: None,
            application_title: None,
        }
    }
}

pub struct Engine {
    state_stack: StateStack,
    ecs: Ecs,
    system_bundles: Vec<SystemBundle>,
    graphics: Option<Graphics>,
    application_title: String,
    asset_store: AssetStore,
}

fn create_ecs() -> Ecs {
    const KEYMAP_FILE: &'static str = "keymap.json";
    let mut ecs = Ecs::new();
    ecs.insert_shared_resource(InputState::new(
        Keymap::from_file(KEYMAP_FILE).unwrap_or(Keymap::default()),
    ));
    ecs
}

impl Engine {
    pub fn new(settings: EngineSettings) -> Engine {
        let mut asset_manager = AssetStore::new();
        asset_manager.load_assets_metadata().unwrap();
        asset_manager.register_loaders(Graphics::loaders());

        Self {
            state_stack: StateStack::new(),
            ecs: create_ecs(),
            system_bundles: vec![],
            graphics: settings.graphics,
            application_title: settings
                .application_title
                .unwrap_or("tuber Application".into()),
            asset_store: asset_manager,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.state_stack.current_state().is_none()
    }

    pub fn initialize_graphics(&mut self, window: Window, window_size: (u32, u32)) {
        if let Some(graphics) = &mut self.graphics {
            graphics.initialize(
                Window(Box::new(&window)),
                window_size,
                &mut self.asset_store,
            );
        }
    }

    pub fn application_title(&self) -> &str {
        &self.application_title
    }

    pub fn push_initial_state(&mut self, state: Box<dyn State>) {
        self.state_stack.push_state(
            state,
            &mut StateContext {
                ecs: &mut self.ecs,
                system_bundles: &mut self.system_bundles,
                asset_store: &mut self.asset_store,
            },
        )
    }

    pub fn step(&mut self, delta_time: f64) {
        let mut state_context = StateContext {
            ecs: &mut self.ecs,
            system_bundles: &mut self.system_bundles,
            asset_store: &mut self.asset_store,
        };
        self.state_stack
            .update_current_state(delta_time, &mut state_context);
    }

    pub fn handle_input(&mut self, input: input::Input) {
        let mut state_context = StateContext {
            ecs: &mut self.ecs,
            system_bundles: &mut self.system_bundles,
            asset_store: &mut self.asset_store,
        };
        self.state_stack.handle_input(input, &mut state_context);
    }

    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        self.graphics
            .as_mut()
            .expect("No graphics")
            .on_window_resized(width, height);
    }

    pub fn render(&mut self) {
        if let Some(graphics) = self.graphics.as_mut() {
            graphics.render_scene(&self.ecs, &mut self.asset_store);
        }
    }
}

pub struct EngineContext {
    pub delta_time: f64,
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
