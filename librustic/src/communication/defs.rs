use crate::communication::{
    protocol::Properties,
    uci::{cmd_in::UciIn, cmd_out::UciOut},
    xboard::{cmd_in::XBoardIn, cmd_out::XBoardOut},
};
use crate::search::defs::SearchReport;
use std::sync::mpsc::Sender;

pub trait IComm {
    fn init(&mut self, cmd_in_tx: Sender<EngineInput>);
    fn properties(&self) -> &Properties;
    fn send(&self, msg: EngineOutput);
    fn shutdown(&mut self);
}

#[derive(Clone)]
pub struct EngineInfo {
    pub name: String,
    pub version: String,
    pub author: String,
}

impl EngineInfo {
    pub fn new(name: String, version: String, author: String) -> Self {
        Self {
            name,
            version,
            author,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum EngineInput {
    Uci(UciIn),
    XBoard(XBoardIn),
    Search(SearchReport),
}

#[derive(PartialEq, Eq)]
pub enum EngineOutput {
    Uci(UciOut),
    XBoard(XBoardOut),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EngineState {
    UciNotUsed,
    Observing,

    Thinking,
    Analyzing,
}

impl std::fmt::Display for EngineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineState::UciNotUsed => write!(f, "UciNotUsed"),
            EngineState::Observing => write!(f, "Observing"),
            EngineState::Thinking => write!(f, "Thinking"),
            EngineState::Analyzing => write!(f, "Analyzing"),
        }
    }
}
