use crate::communication::defs::EngineInput;
use crate::communication::xboard::XBoard;
use std::sync::mpsc::Sender;

impl XBoard {
    pub fn input_thread(&mut self, transmit_to_engine: Sender<EngineInput>) {}
}
