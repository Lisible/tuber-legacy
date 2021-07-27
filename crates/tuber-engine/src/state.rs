use tuber_core::input::{Input, InputState};
use tuber_core::DeltaTime;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;

pub trait State {
    fn initialize(&mut self, _state_context: &mut StateContext) {}
    fn update(&mut self, _state_context: &mut StateContext) {}
    fn stack_requests(&mut self) -> Vec<StateStackRequest> {
        vec![]
    }
}

pub struct StateStack {
    states: Vec<Box<dyn State>>,
    state_context: Vec<StateContext>,
}

impl StateStack {
    pub fn new() -> Self {
        Self {
            states: vec![],
            state_context: vec![],
        }
    }

    pub fn push_state(&mut self, state: Box<dyn State>) {
        let mut state = state;
        let mut state_context = StateContext::default();
        state.initialize(&mut state_context);

        if let Some(state_context) = self.state_context.last_mut() {
            let mut input_state = state_context
                .ecs
                .shared_resource_mut::<InputState>()
                .expect("Input state");
            input_state.clear();
        }

        self.states.push(state);
        self.state_context.push(state_context);
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
        self.state_context.pop();
    }

    pub fn current_state(&self) -> Option<&Box<dyn State>> {
        self.states.last()
    }

    pub fn current_state_mut(&mut self) -> Option<&mut Box<dyn State>> {
        self.states.last_mut()
    }

    pub fn current_state_context_mut(&mut self) -> Option<&mut StateContext> {
        self.state_context.last_mut()
    }

    pub fn state_contexts_mut(&mut self) -> &mut Vec<StateContext> {
        &mut self.state_context
    }

    pub fn update_current_state(&mut self, delta_time: f64) {
        let state_context = self
            .state_context
            .last_mut()
            .expect("Expected current state's context");
        state_context
            .ecs
            .insert_shared_resource(DeltaTime(delta_time));
        let state = self.states.last_mut().expect("Expected current state");
        state.update(state_context);
        for system_bundle in &mut state_context.system_bundles {
            system_bundle.step(&mut state_context.ecs).unwrap();
        }

        let mut reqs = state.stack_requests();
        reqs.reverse();
        while let Some(req) = reqs.pop() {
            self.handle_request(req);
        }
    }

    pub fn handle_input(&mut self, input: Input) {
        let state_context = self
            .state_context
            .last_mut()
            .expect("Expected current state's context");
        let mut input_state = state_context
            .ecs
            .shared_resource_mut::<InputState>()
            .expect("Input state");
        input_state.handle_input(input);
    }

    pub fn handle_request(&mut self, request: StateStackRequest) {
        match request {
            StateStackRequest::Pop => self.pop_state(),
            StateStackRequest::Push(state) => self.push_state(state),
        }
    }
}

pub enum StateStackRequest {
    Pop,
    Push(Box<dyn State>),
}

pub struct StateContext {
    pub ecs: Ecs,
    pub system_bundles: Vec<SystemBundle>,
}

impl Default for StateContext {
    fn default() -> Self {
        let mut ecs = Ecs::new();
        ecs.insert_shared_resource(InputState::new());
        Self {
            ecs,
            system_bundles: vec![],
        }
    }
}
