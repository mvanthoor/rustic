use crate::engine::Engine;
use librustic::communication::defs::EngineState;

impl Engine {
    pub fn get_state(&self) -> &EngineState {
        &self.state
    }

    pub fn is_observing(&self) -> bool {
        self.state == EngineState::Observing
    }

    pub fn is_waiting(&self) -> bool {
        self.state == EngineState::Waiting
    }

    pub fn is_thinking(&self) -> bool {
        self.state == EngineState::Thinking
    }

    pub fn is_analyzing(&self) -> bool {
        self.state == EngineState::Analyzing
    }

    pub fn set_observing(&mut self) {
        self.state = EngineState::Observing;
    }

    pub fn set_waiting(&mut self) {
        self.state = EngineState::Waiting;
    }

    pub fn set_thinking(&mut self) {
        self.state = EngineState::Thinking;
    }

    pub fn set_analyzing(&mut self) {
        self.state = EngineState::Analyzing;
    }
}
