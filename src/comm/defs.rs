/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2022, Marcel Vanthoor
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

use crate::{
    board::Board,
    engine::defs::{EngineOption, EngineState, GameResult, Information},
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

pub use crate::comm::protocols::{
    uci::{Uci, UciIn, UciOut},
    xboard::{TimeControl, XBoard, XBoardIn, XBoardOut},
};

// These are the types of communication the engine is capable of.
pub struct CommType;

impl CommType {
    pub const XBOARD: &'static str = "xboard";
    pub const UCI: &'static str = "uci";
}

pub struct CommInfo {
    protocol_name: &'static str,
    supports_fancy_about: bool,
    requires_stateful_mode: bool,
    requires_game_result: bool,
    startup_state: EngineState,
}

impl CommInfo {
    pub fn new(
        protocol_name: &'static str,
        supports_fancy_about: bool,
        requires_stateful_mode: bool,
        requires_game_result: bool,
        startup_state: EngineState,
    ) -> Self {
        Self {
            protocol_name,
            supports_fancy_about,
            requires_stateful_mode,
            requires_game_result,
            startup_state,
        }
    }

    pub fn protocol_name(&self) -> &str {
        self.protocol_name
    }

    pub fn supports_fancy_about(&self) -> bool {
        self.supports_fancy_about
    }

    pub fn requires_stateful_mode(&self) -> bool {
        self.requires_stateful_mode
    }

    pub fn requires_game_result(&self) -> bool {
        self.requires_game_result
    }

    pub fn startup_state(&self) -> EngineState {
        self.startup_state
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
    fn send(&self, msg: CommOut);
    fn shutdown(&mut self);
    fn info(&self) -> &CommInfo;
}

#[derive(PartialEq, Clone)]
pub enum CommIn {
    Uci(UciIn),
    XBoard(XBoardIn),

    // Common incoming commands
    Quit,
    Unknown(String),

    // Custom
    Board,
    History,
    Eval,
    State,
    ClearTt,
    Help,

    // Ignore an incoming command on purpose
    Ignore(String),
}

pub enum CommOut {
    Uci(UciOut),
    XBoard(XBoardOut),

    // Common output for all protocols
    BestMove(Move, Option<GameResult>), // Transmit the engine's best move and result.
    SearchCurrMove(SearchCurrentMove),  // Transmit currently considered move.
    SearchSummary(SearchSummary),       // Transmit search information.
    SearchStats(SearchStats),           // Transmit search Statistics.
    Message(String),                    // Transmits a message to the GUI.
    Error(&'static str, String),        // Transmits an error message.
    Quit,                               // Terminates the output thread.

    // Output to screen when running in a terminal window.
    PrintBoard,
    PrintHistory,
    PrintHelp,
    PrintState(EngineState),
    PrintEval(i16, i16),
}
