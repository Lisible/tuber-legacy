use tuber_core::input::Input;
use tuber_core::DeltaTime;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;

use crate::engine_context::EngineContext;

pub trait State {
    fn initialize(
        &mut self,
        _ecs: &mut Ecs,
        _system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        _engine_context: &mut EngineContext,
    ) {
    }
    fn update(&mut self, _ecs: &mut Ecs, _engine_context: &mut EngineContext) {}
    fn render(&mut self, _ecs: &mut Ecs, _engine_context: &mut EngineContext) {}
    fn stack_requests(&mut self) -> Vec<StateStackRequest> {
        vec![]
    }
}

pub struct StateStack {
    initial_state: Option<Box<dyn State>>,
    states: Vec<Box<dyn State>>,
}

impl StateStack {
    #[must_use]
    pub fn new(initial_state: Option<Box<dyn State>>) -> Self {
        Self {
            initial_state,
            states: vec![],
        }
    }

    pub fn push_initial_state(
        &mut self,
        ecs: &mut Ecs,
        system_bundle: &mut Vec<SystemBundle<EngineContext>>,
        engine_context: &mut EngineContext,
    ) {
        let initial_state = self.initial_state.take().expect("No initial state");
        self.push_state(initial_state, ecs, system_bundle, engine_context);
    }

    pub fn push_state(
        &mut self,
        state: Box<dyn State>,
        ecs: &mut Ecs,
        system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        engine_context: &mut EngineContext,
    ) {
        let mut state = state;
        state.initialize(ecs, system_bundles, engine_context);

        self.states.push(state);
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    #[allow(clippy::borrowed_box)]
    #[must_use]
    pub fn current_state(&self) -> Option<&Box<dyn State>> {
        self.states.last()
    }

    pub fn current_state_mut(&mut self) -> Option<&mut Box<dyn State>> {
        self.states.last_mut()
    }

    pub fn update_current_state<'a>(
        &mut self,
        delta_time: f64,
        ecs: &mut Ecs,
        system_bundles: &'a mut Vec<SystemBundle<EngineContext>>,
        engine_context: &'a mut EngineContext,
    ) {
        ecs.insert_shared_resource(DeltaTime(delta_time));
        let state = self.states.last_mut().expect("Expected current state");
        state.update(ecs, engine_context);

        for system_bundle in system_bundles.iter_mut() {
            system_bundle.step(ecs, engine_context).unwrap();
        }

        let mut reqs = state.stack_requests();
        reqs.reverse();
        while let Some(req) = reqs.pop() {
            self.handle_request(req, ecs, system_bundles, engine_context);
        }
    }

    pub fn render_current_state<'a>(
        &mut self,
        ecs: &mut Ecs,
        engine_context: &'a mut EngineContext,
    ) {
        let state = self.states.last_mut().expect("Expected current state");
        state.render(ecs, engine_context);
    }

    #[allow(clippy::unused_self)]
    pub fn handle_input(&mut self, input: &Input, engine_context: &mut EngineContext) {
        engine_context.input_state.handle_input(input);
    }

    pub fn handle_request(
        &mut self,
        request: StateStackRequest,
        ecs: &mut Ecs,
        system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        engine_context: &mut EngineContext,
    ) {
        match request {
            StateStackRequest::Pop => self.pop_state(),
            StateStackRequest::Push(state) => {
                self.push_state(state, ecs, system_bundles, engine_context);
            }
        }
    }
}

pub enum StateStackRequest {
    Pop,
    Push(Box<dyn State>),
}
