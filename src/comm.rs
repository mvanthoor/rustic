/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

mod shared;
mod uci;
mod xboard;

use crate::{
    board::Board,
    engine::defs::{EngineOption, Information},
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

pub use uci::{Uci, UciInput, UciOutput};
pub use xboard::{XBoard, XBoardInput, XBoardOutput};

// These are the types of communication the engine is capable of.
pub struct CommType {
    protocol: String,
    stateful: bool,
    fancy_about: bool,
}

impl CommType {
    pub const UCI: &'static str = "uci";
    pub const XBOARD: &'static str = "xboard";

    pub fn protocol(&self) -> String {
        self.protocol.clone()
    }

    pub fn stateful(&self) -> bool {
        self.stateful
    }

    pub fn fancy_about(&self) -> bool {
        self.fancy_about
    }
}

// Defines the public functions a Comm module must implement.
pub trait IComm {
    fn init(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<EngineOption>>,
    );
    fn send(&self, msg: CommOutput);
    fn wait_for_shutdown(&mut self);
    fn info(&self) -> CommType;
}

#[derive(PartialEq, Clone)]
pub enum CommInput {
    Uci(UciInput),
    XBoard(XBoardInput),

    // Common incoming commands
    Quit,
    Ok,
    Unknown(String),

    // Custom
    Board,
    History,
    Eval,
    Help,
}

pub enum CommOutput {
    Uci(UciOutput),
    XBoard(XBoardOutput),

    // Common output for all protocols
    BestMove(Move),                    // Transmit the engine's best move.
    SearchCurrMove(SearchCurrentMove), // Transmit currently considered move.
    SearchSummary(SearchSummary),      // Transmit search information.
    SearchStats(SearchStats),          // Transmit search Statistics.
    Message(String),                   // Transmits a message to the GUI
    Error(String, String),             // Transmit an error to the GUI
    Quit,                              // Terminates the output thread.

    // Output to screen when running in a terminal window.
    PrintBoard,
    PrintHistory,
    PrintHelp,
    PrintEval(i16, i16),
}
