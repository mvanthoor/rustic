use crate::{
    board::Board,
    communication::protocol::Properties,
    communication::uci::uci_option::UciOption,
    communication::uci::{cmd_in::UciIn, cmd_out::UciOut},
    search::defs::SearchReport,
};
use std::sync::{mpsc::Sender, Arc, Mutex};

pub trait IComm {
    fn init(
        &mut self,
        cmd_in_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<UciOption>>,
    );
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
    Observing,
    Waiting,
    Thinking,
    Analyzing,
}
