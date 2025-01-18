use crate::{
    communication::feature::Feature,
    communication::protocol::Properties,
    communication::uci::{cmd_in::UciIn, cmd_out::UciOut},
    communication::xboard::cmd_in::XBoardIn,
    search::defs::SearchReport,
};
use std::sync::{mpsc::Sender, Arc};

pub trait IComm {
    fn init(&mut self, cmd_in_tx: Sender<EngineInput>, options: Arc<Vec<Feature>>);
    fn properties(&self) -> &Properties;
    fn send(&self, msg: UciOut);
    fn shutdown(&mut self);
}

pub enum EngineInput {
    Uci(UciIn),
    XBoard(XBoardIn),
    Search(SearchReport),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EngineState {
    UciNotUsed,
    Observing,
    Waiting,
    Thinking,
    Analyzing,
}
