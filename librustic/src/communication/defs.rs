use crate::{
    communication::feature::Feature,
    communication::protocol::Properties,
    communication::uci::{cmd_in::UciIn, cmd_out::UciOut},
    search::defs::SearchReport,
};
use std::sync::{mpsc::Sender, Arc};

pub trait IComm {
    fn init(&mut self, cmd_in_tx: Sender<Information>, options: Arc<Vec<Feature>>);
    fn properties(&self) -> &Properties;
    fn send(&self, msg: UciOut);
    fn shutdown(&mut self);
}

pub enum Information {
    Command(UciIn),
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
