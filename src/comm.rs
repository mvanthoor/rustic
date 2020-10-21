pub mod console;
pub mod uci;
// pub mod xboard;

use crate::{
    board::Board, engine::defs::Information, misc::parse, movegen::defs::Move,
    search::defs::SearchSummary,
};
use console::ConsoleReport;
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};
use uci::UciReport;

// These are the types of communication the engine is capable of.
pub struct CommType;
impl CommType {
    // pub const XBOARD: &'static str = "xboard";
    pub const UCI: &'static str = "uci";
    pub const CONSOLE: &'static str = "console";
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
    // Reaction of engine to incoming commands.
    Update,   // Request Comm module to update its state.
    Quit,     // Quit the Comm module.
    Identify, // Transmit identification of the engine.
    Ready,    // Transmit that the engine is ready.

    // Output to screen for custom commands.
    // Mostly used for running in a terminal window.
    PrintHelp,
    PrintBoard,
    PrintEvaluation(i16),
    PrintMessage(String),
    PrintBestMove(Move),
    PrintSearchSummary(SearchSummary),
}

#[derive(PartialEq, Clone)]
pub enum GeneralReport {
    Nothing,
    Unknown,
    Help,
    Quit,
}

// These are the commands a Comm module can create and send back to the
// engine in the main thread.
#[derive(PartialEq, Clone)]
pub enum CommReport {
    General(GeneralReport),
    Console(ConsoleReport),
    Uci(UciReport),
}

impl CommReport {
    pub fn is_valid(&self) -> bool {
        // Match the incoming command.
        match self {
            // Check if squares and promotion piece actually exist.
            Self::Console(ConsoleReport::Move(m)) => {
                parse::algebraic_move_to_number(&m[..]).is_ok()
            }
            _ => true,
        }
    }
}
