use tuber_core::asset::AssetStore;
use tuber_core::input::{Input, InputState};
use tuber_core::transform::Transform2D;
use tuber_core::DeltaTime;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;
use tuber_graphics::camera::{Active, OrthographicCamera};

pub trait State {
    fn initialize(&mut self, state_context: &mut StateContext) {
        state_context.ecs.insert((
            OrthographicCamera {
                left: 0.0,
                right: 800.0,
                top: 0.0,
                bottom: 600.0,
                near: -100.0,
                far: 100.0,
            },
            Active,
            Transform2D::default(),
        ));
    }
    fn update(&mut self, _state_context: &mut StateContext) {}
    fn stack_requests(&mut self) -> Vec<StateStackRequest> {
        vec![]
    }
}

pub struct StateStack {
    states: Vec<Box<dyn State>>,
}

impl StateStack {
    pub fn new() -> Self {
        Self { states: vec![] }
    }

    pub fn push_state(&mut self, state: Box<dyn State>, state_context: &mut StateContext) {
        let mut state = state;
        state.initialize(state_context);

        self.states.push(state);
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    pub fn current_state(&self) -> Option<&Box<dyn State>> {
        self.states.last()
    }

    pub fn current_state_mut(&mut self) -> Option<&mut Box<dyn State>> {
        self.states.last_mut()
    }

    pub fn update_current_state(&mut self, delta_time: f64, state_context: &mut StateContext) {
        state_context
            .ecs
            .insert_shared_resource(DeltaTime(delta_time));
        let state = self.states.last_mut().expect("Expected current state");
        state.update(state_context);
        for system_bundle in &mut *state_context.system_bundles {
            system_bundle.step(&mut state_context.ecs, &mut ()).unwrap();
        }

        let mut reqs = state.stack_requests();
        reqs.reverse();
        while let Some(req) = reqs.pop() {
            self.handle_request(req, state_context);
        }
    }

    pub fn handle_input(&mut self, input: Input, state_context: &mut StateContext) {
        let mut input_state = state_context
            .ecs
            .shared_resource_mut::<InputState>()
            .expect("Input state");
        input_state.handle_input(input);
    }

    pub fn handle_request(&mut self, request: StateStackRequest, state_context: &mut StateContext) {
        match request {
            StateStackRequest::Pop => self.pop_state(),
            StateStackRequest::Push(state) => self.push_state(state, state_context),
        }
    }
}

pub enum StateStackRequest {
    Pop,
    Push(Box<dyn State>),
}

pub struct StateContext<'engine> {
    pub ecs: &'engine mut Ecs,
    pub system_bundles: &'engine mut Vec<SystemBundle<()>>,
    pub asset_store: &'engine mut AssetStore,
}
