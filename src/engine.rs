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

mod about;
mod comm_reports;
pub mod defs;
mod hash_table;
mod main_loop;
mod search_reports;
mod utils;

use crate::{
    board::Board,
    comm::{uci::Uci, CommControl, CommType, IComm},
    defs::EngineRunResult,
    engine::defs::{ErrFatal, Information, Settings},
    misc::{cmdline::CmdLine, perft},
    movegen::MoveGenerator,
    search::{defs::SearchControl, Search},
};
use crossbeam_channel::Receiver;
use hash_table::{HashTable, PerftData, SearchData};
use std::sync::{Arc, Mutex};

#[cfg(feature = "extra")]
use crate::{
    board::defs::Pieces,
    extra::{testsuite, wizardry},
};

// This struct holds the chess engine and its functions, so they are not
// all seperate entities in the global space.
pub struct Engine {
    quit: bool,                                     // Flag that will quit the main thread.
    settings: Settings,                             // Struct holding all the settings.
    cmdline: CmdLine,                               // Command line interpreter.
    comm: Box<dyn IComm>,                           // Communications (active).
    board: Arc<Mutex<Board>>,                       // This is the main engine board.
    hash_perft: Arc<Mutex<HashTable<PerftData>>>,   // Hash table for running perft.
    hash_search: Arc<Mutex<HashTable<SearchData>>>, // Hash table for search information.
    mg: Arc<MoveGenerator>,                         // Move Generator.
    info_rx: Option<Receiver<Information>>,         // Receiver for incoming information.
    search: Search,                                 // Search object (active).
    tmp_no_xboard: bool,                            // Temporary variable to disable xBoard
}

impl Engine {
    // Create e new engine.
    pub fn new() -> Self {
        // Create the command-line object.
        let cmdline = CmdLine::new();
        let mut is_xboard = false;

        // Create the communication interface
        let comm: Box<dyn IComm> = match &cmdline.comm()[..] {
            CommType::XBOARD => {
                is_xboard = true;
                Box::new(Uci::new())
            }
            CommType::UCI => Box::new(Uci::new()),
            _ => panic!(ErrFatal::CREATE_COMM),
        };

        // Get engine settings from the command-line
        let threads = cmdline.threads();
        let quiet = cmdline.has_quiet();

        let s1 = if cmdline.perft() > 0 { 64 } else { 0 }; // Perft Hash in MB
        let s2 = if cmdline.perft() == 0 { 256 } else { 0 }; // Search Hash in MB

        let hash_perft = Arc::new(Mutex::new(HashTable::<PerftData>::new(s1)));
        let hash_search = Arc::new(Mutex::new(HashTable::<SearchData>::new(s2)));

        // Create the engine itself.
        Self {
            quit: false,
            settings: Settings { threads, quiet },
            cmdline,
            comm,
            board: Arc::new(Mutex::new(Board::new())),
            mg: Arc::new(MoveGenerator::new()),
            hash_perft,
            hash_search,
            info_rx: None,
            search: Search::new(),
            tmp_no_xboard: is_xboard,
        }
    }

    // Run the engine.
    pub fn run(&mut self) -> EngineRunResult {
        // This is temporary. Quit the engine immediately if anyone tries
        // to start it in XBoard mode, as this is not implemented yet.
        if self.tmp_no_xboard {
            return Err(7);
        }

        self.print_ascii_logo();
        self.print_about();
        self.print_settings(self.settings.threads, self.comm.get_protocol_name());
        println!();

        // Setup position and abort if this fails.
        self.setup_position()?;

        // Run a specific action if requested...
        let mut action_requested = false;

        // Run perft if requested.
        if self.cmdline.perft() > 0 {
            action_requested = true;
            perft::run(self.board.clone(), self.cmdline.perft(), self.mg.clone());
        }

        // === Only available with "extra" features enabled. ===
        #[cfg(feature = "extra")]
        // Generate magic numbers if requested.
        if self.cmdline.has_wizardry() {
            action_requested = true;
            wizardry::find_magics(Pieces::ROOK);
            wizardry::find_magics(Pieces::BISHOP);
        };

        #[cfg(feature = "extra")]
        // Run large EPD test suite if requested.
        if self.cmdline.has_test() {
            action_requested = true;
            testsuite::run();
        }
        // =====================================================

        // In the main loop, the engine manages its resources so it will be
        // able to play legal chess and communicate with different user
        // interfaces.
        if !action_requested {
            self.main_loop();
        }

        // There are three ways to exit the engine: when the FEN-setup
        // fails, because of a crash, or normally. In the first two cases,
        // this Ok(()) won't be reached.
        Ok(())
    }

    // This function quits Commm, Search, and then the engine thread itself.
    pub fn quit(&mut self) {
        self.search.send(SearchControl::Quit);
        self.comm.send(CommControl::Quit);
        self.quit = true;
    }
}
