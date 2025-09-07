use crate::communication::{
    protocol::Properties,
    uci::{cmd_in::UciIn, cmd_out::UciOut},
    xboard::{cmd_in::XBoardIn, cmd_out::XBoardOut},
};
use crate::search::defs::SearchReport;
use std::{
    fmt::{self, Display, Formatter},
    sync::mpsc::Sender,
};

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

pub enum EngineInput {
    Uci(UciIn),
    XBoard(XBoardIn),
    Search(SearchReport),
}

pub enum EngineOutput {
    Uci(UciOut),
    XBoard(XBoardOut),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EngineState {
    UciNotUsed,
    Observing,
    Waiting,
    Thinking,
    Analyzing,
}

impl Display for EngineState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let printable = match *self {
            EngineState::UciNotUsed => "UCI has no states",
            EngineState::Observing => "Observing",
            EngineState::Waiting => "Waiting",
            EngineState::Thinking => "Thinking",
            EngineState::Analyzing => "Analyzing",
        };
        write!(f, "{printable}")
    }
}
