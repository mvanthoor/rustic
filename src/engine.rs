mod about;
mod comm_handler;
pub mod defs;
mod game_result;
mod main_loop;
mod search_handler;
mod transposition;
mod utils;

use crate::{
    board::Board,
    comm::defs::{CommOut, CommType, IComm, Uci, XBoard},
    defs::EngineRunResult,
    engine::defs::{
        EngineOption, EngineOptionDefaults, EngineSetOption, ErrFatal, Information, Settings,
        UiElement, Verbosity,
    },
    misc::{cmdline::CmdLine, perft},
    movegen::MoveGenerator,
    search::{defs::SearchControl, Search},
};
use crossbeam_channel::Receiver;
use std::sync::{Arc, Mutex};
use transposition::{PerftData, SearchData, TT};

#[cfg(feature = "extra")]
use crate::{
    board::defs::Pieces,
    extra::{testsuite, wizardry},
};

use self::defs::EngineState;

// This struct holds the chess engine and its functions, so they are not
// all separate entities in the global space.
pub struct Engine {
    quit: bool,                             // Flag that will quit the main thread.
    state: EngineState,                     // Keeps the current engine activity.
    settings: Settings,                     // Struct holding all the settings.
    options: Arc<Vec<EngineOption>>,        // Engine options exported to the GUI.
    cmdline: CmdLine,                       // Command line interpreter.
    comm: Box<dyn IComm>,                   // Communications (active).
    board: Arc<Mutex<Board>>,               // This is the main engine board.
    tt_perft: Arc<Mutex<TT<PerftData>>>,    // TT for running perft.
    tt_search: Arc<Mutex<TT<SearchData>>>,  // TT for search information.
    mg: Arc<MoveGenerator>,                 // Move Generator.
    info_rx: Option<Receiver<Information>>, // Receiver for incoming information.
    search: Search,                         // Search object (active).
}

impl Engine {
    // Create e new engine.
    pub fn new() -> Self {
        // Determine if the compiled engine is 32 or 64-bit
        let is_64_bit = std::mem::size_of::<usize>() == 8;

        // Create the command-line object.
        let cmdline = CmdLine::new();

        // Create the communication interface
        let comm: Box<dyn IComm> = match cmdline.comm().as_str() {
            CommType::XBOARD => Box::new(XBoard::new()),
            CommType::UCI => Box::new(Uci::new()),
            _ => panic!("{}", ErrFatal::CREATE_COMM),
        };

        // Get engine settings from the command-line.
        let threads = cmdline.threads();
        let verbosity = if cmdline.has_quiet() {
            Verbosity::Quiet
        } else {
            Verbosity::Full
        };
        let tt_size = cmdline.hash();
        let tt_max = if is_64_bit {
            EngineOptionDefaults::HASH_MAX_64_BIT
        } else {
            EngineOptionDefaults::HASH_MAX_32_BIT
        };

        // List of options that should be announced to the GUI.
        let options = vec![
            EngineOption::new(
                EngineSetOption::HASH,
                UiElement::Spin,
                Some(EngineOptionDefaults::HASH_DEFAULT.to_string()),
                Some(EngineOptionDefaults::HASH_MIN.to_string()),
                Some(tt_max.to_string()),
            ),
            EngineOption::new(
                EngineSetOption::CLEAR_HASH,
                UiElement::Button,
                None,
                None,
                None,
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
            state: comm.info().startup_state(),
            settings: Settings {
                threads,
                verbosity,
                tt_size,
            },
            options: Arc::new(options),
            cmdline,
            comm,
            board: Arc::new(Mutex::new(Board::new())),
            mg: Arc::new(MoveGenerator::new()),
            tt_perft,
            tt_search,
            info_rx: None,
            search: Search::new(),
        }
    }

    // Run the engine.
    pub fn run(&mut self) -> EngineRunResult {
        if self.comm.info().supports_fancy_about() {
            self.print_fancy_about(&self.settings, self.comm.info().protocol_name());
        } else {
            self.print_simple_about(&self.settings, self.comm.info().protocol_name());
        }

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

    // This function quits Comm, Search, and then the engine thread itself.
    pub fn quit(&mut self) {
        self.search.send(SearchControl::Quit);
        self.comm.send(CommOut::Quit);
        self.quit = true;
    }

    fn is_observing(&self) -> bool {
        self.state == EngineState::Observing
    }

    fn is_waiting(&self) -> bool {
        self.state == EngineState::Waiting
    }

    fn is_thinking(&self) -> bool {
        self.state == EngineState::Thinking
    }

    fn is_analyzing(&self) -> bool {
        self.state == EngineState::Analyzing
    }

    fn set_observing(&mut self) {
        self.state = EngineState::Observing;
    }

    fn set_waiting(&mut self) {
        self.state = EngineState::Waiting;
    }

    fn set_thinking(&mut self) {
        self.state = EngineState::Thinking;
    }

    fn set_analyzing(&mut self) {
        self.state = EngineState::Analyzing;
    }
}
