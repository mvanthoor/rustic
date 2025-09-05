use crate::communication::{
    feature::Feature,
    protocol::Properties,
    uci::{cmd_in::UciIn, cmd_out::UciOut},
    xboard::{cmd_in::XBoardIn, cmd_out::XBoardOut},
};
use crate::search::defs::SearchReport;
use std::sync::{Arc, mpsc::Sender};

pub trait IComm {
    fn init(&mut self, cmd_in_tx: Sender<EngineInput>, options: Arc<Vec<Feature>>);
    fn properties(&self) -> &Properties;
    fn send(&self, msg: EngineOutput);
    fn shutdown(&mut self);
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
