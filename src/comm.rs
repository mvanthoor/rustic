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

pub mod uci;
pub mod xboard;

use crate::{
    board::Board,
    engine::defs::{EngineOption, EngineSetOption, Information},
    movegen::defs::Move,
    search::defs::{GameTime, SearchCurrentMove, SearchStats, SearchSummary},
};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

// These are the types of communication the engine is capable of.
pub struct CommType;
impl CommType {
    pub const XBOARD: &'static str = "xboard";
    pub const UCI: &'static str = "uci";
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
    fn get_protocol_name(&self) -> &'static str;
}

#[derive(PartialEq)]

pub enum UciOutput {
    Identify, // Transmit identification of the engine.
    Ready,    // Transmit that the engine is ready.
}

pub enum XBoardOutput {
    Pong(u8),
}

pub enum CommOutput {
    Uci(UciOutput),
    XBoard(XBoardOutput),

    // Common output for all protocols
    BestMove(Move),                    // Transmit the engine's best move.
    SearchCurrMove(SearchCurrentMove), // Transmit currently considered move.
    SearchSummary(SearchSummary),      // Transmit search information.
    SearchStats(SearchStats),          // Transmit search Statistics.
    InfoString(String),                // Transmit general information.
    Quit,                              // Terminates the output thread.

    // Output to screen when running in a terminal window.
    PrintBoard,
    PrintHistory,
    PrintHelp,
}

// This is the list of commands the engine understands. Information coming
// in through the Comm module is turned into one of these commands which
// will then be sent to the engine thread.
#[derive(PartialEq, Clone)]
pub enum UciInput {
    Identification,
    NewGame,
    IsReady,
    SetOption(EngineSetOption),
    Position(String, Vec<String>),
    GoInfinite,
    GoDepth(i8),
    GoMoveTime(u128),
    GoNodes(usize),
    GoGameTime(GameTime),
    Stop,
    Quit,

    // Custom
    Board,
    History,
    Eval,
    Help,

    // Empty or unknown command.
    Unknown,
}

#[derive(PartialEq, Clone)]
pub enum XBoardInput {
    Ping(u8),
    Quit,

    // Custom commands
    Board,
    History,
    Eval,
    Help,
    Unknown,
}

#[derive(PartialEq, Clone)]
pub enum CommInput {
    Uci(UciInput),
    XBoard(XBoardInput),
}
