pub mod uci;
// pub mod xboard;

use crate::{
    board::Board, engine::defs::Information, movegen::defs::Move, search::defs::SearchSummary,
};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};
use uci::UciReport;

// These are the types of communication the engine is capable of.
pub struct CommType;
impl CommType {
    // pub const XBOARD: &'static str = "xboard";
    pub const UCI: &'static str = "uci";
}

// Defines the public functions a Comm module must implement.
pub trait IComm {
    fn init(&mut self, report_tx: Sender<Information>, board: Arc<Mutex<Board>>);
    fn send(&self, msg: CommControl);
    fn wait_for_shutdown(&mut self);
    fn get_protocol_name(&self) -> &'static str;
}

#[derive(PartialEq)]
pub enum CommControl {
    // Reactions of engine to incoming commands.
    Update,   // Request Comm module to update its state.
    Quit,     // Quit the Comm module.
    Identify, // Transmit identification of the engine.
    Ready,    // Transmit that the engine is ready.

    PrintBestMove(Move),
    PrintSearchSummary(SearchSummary),

    // Output to screen when running in a terminal window.
    PrintMessage(String),
    PrintBoard,
    PrintHelp,
}

// These are the commands a Comm module can create and send back to the
// engine in the main thread.
#[derive(PartialEq, Clone)]
pub enum CommReport {
    Uci(UciReport),
}

impl CommReport {
    pub fn is_valid(&self) -> bool {
        // Match the incoming command.
        match self {
            _ => true,
        }
    }
}
