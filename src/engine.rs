/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2024, Marcel Vanthoor
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
mod main_loop;
mod search_reports;
mod transposition;
mod utils;

use crate::{
    board::Board,
    comm::{uci::Uci, CommControl, CommType, IComm},
    defs::EngineRunResult,
    engine::defs::{
        EngineOption, EngineOptionDefaults, EngineOptionName, ErrFatal, Information, Settings,
        UiElement,
    },
    misc::{cmdline::CmdLine, perft},
    movegen::{MoveGenerator, defs::Move},
    search::{defs::SearchControl, SearchManager},
};
use crossbeam_channel::Receiver;
use std::sync::{Arc, Mutex};
use transposition::{PerftData, SearchData, TT};

#[cfg(feature = "extra")]
use crate::{
    board::defs::Pieces,
    extra::{testsuite, wizardry},
};

// This struct holds the chess engine and its functions, so they are not
// all seperate entities in the global space.
pub struct Engine {
    quit: bool,                             // Flag that will quit the main thread.
    settings: Settings,                     // Struct holding all the settings.
    options: Arc<Vec<EngineOption>>,        // Engine options exported to the GUI
    cmdline: CmdLine,                       // Command line interpreter.
    comm: Box<dyn IComm>,                   // Communications (active).
    board: Arc<Mutex<Board>>,               // This is the main engine board.
    tt_perft: Arc<Mutex<TT<PerftData>>>,    // TT for running perft.
    tt_search: Arc<Mutex<TT<SearchData>>>,  // TT for search information.
    mg: Arc<MoveGenerator>,                 // Move Generator.
    info_rx: Option<Receiver<Information>>, // Receiver for incoming information.
    search: SearchManager,                  // Search manager (active).
    tmp_no_xboard: bool,                    // Temporary variable to disable xBoard
    pondering: bool,                        // If ponder is active
    delayed_bestmove: Option<Move>,         // 
}

impl Engine {
    // Create e new engine.
    pub fn new() -> Self {
        // Determine if the compiled engine is 32 or 64-bit
        let is_64_bit = std::mem::size_of::<usize>() == 8;

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
            _ => panic!("{}", ErrFatal::CREATE_COMM),
        };

        // Get engine settings from the command-line.
        let threads = cmdline.threads();
        let quiet = cmdline.has_quiet();
        let tt_size = cmdline.hash();
        let tt_max = if is_64_bit {
            EngineOptionDefaults::HASH_MAX_64_BIT
        } else {
            EngineOptionDefaults::HASH_MAX_32_BIT
        };

        // List of options that should be announced to the GUI.
        let options = vec![
            EngineOption::new(
                EngineOptionName::HASH,
                UiElement::Spin,
                Some(EngineOptionDefaults::HASH_DEFAULT.to_string()),
                Some(EngineOptionDefaults::HASH_MIN.to_string()),
                Some(tt_max.to_string()),
            ),
            EngineOption::new(
                EngineOptionName::CLEAR_HASH,
                UiElement::Button,
                None,
                None,
                None,
            ),
            EngineOption::new(
                EngineOptionName::SHARP_MARGIN,
                UiElement::Spin,
                Some(EngineOptionDefaults::SHARP_MARGIN_DEFAULT.to_string()),
                Some(EngineOptionDefaults::SHARP_MARGIN_MIN.to_string()),
                Some(EngineOptionDefaults::SHARP_MARGIN_MAX.to_string()),
            ),
        ];

        // Initialize correct TT.
        let tt_perft: Arc<Mutex<TT<PerftData>>>;
        let tt_search: Arc<Mutex<TT<SearchData>>>;
        if cmdline.perft() > 0 {
            tt_perft = Arc::new(Mutex::new(TT::<PerftData>::new(tt_size)));
            tt_search = Arc::new(Mutex::new(TT::<SearchData>::new(0)));
        } else {
            tt_perft = Arc::new(Mutex::new(TT::<PerftData>::new(0)));
            tt_search = Arc::new(Mutex::new(TT::<SearchData>::new(tt_size)));
        };

        // Create the engine itself.
        Self {
            quit: false,
            settings: Settings {
                threads,
                quiet,
                tt_size,
                sharp_margin: EngineOptionDefaults::SHARP_MARGIN_DEFAULT,
            },
            options: Arc::new(options),
            cmdline,
            comm,
            board: Arc::new(Mutex::new(Board::new())),
            mg: Arc::new(MoveGenerator::new()),
            tt_perft,
            tt_search,
            info_rx: None,
            search: SearchManager::new(threads),
            tmp_no_xboard: is_xboard,
            pondering: false,
            delayed_bestmove: None,
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
        self.print_about(&self.settings);
        println!();

        // Setup position and abort if this fails.
        self.setup_position()?;

        // Run a specific action if requested...
        let mut action_requested = false;

        // Run perft if requested.
        if self.cmdline.perft() > 0 {
            action_requested = true;
            perft::run(
                self.board.clone(),
                self.cmdline.perft(),
                Arc::clone(&self.mg),
                Arc::clone(&self.tt_perft),
                self.settings.tt_size > 0,
            );
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
        // Run large EPD test suite if requested. Because the -p (perft)
        // option is not used in this scenario, the engine initializes the
        // search TT instead of the one for perft. The -e option is
        // not available in a non-extra compilation, so it cannot be
        // checked there. Just fix the issue by resizing both the perft and
        // search TT's appropriately for running the EPD suite.
        if self.cmdline.has_test() {
            action_requested = true;
            self.tt_perft
                .lock()
                .expect(ErrFatal::LOCK)
                .resize(self.settings.tt_size);
            self.tt_search.lock().expect(ErrFatal::LOCK).resize(0);
            testsuite::run(Arc::clone(&self.tt_perft), self.settings.tt_size > 0);
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
