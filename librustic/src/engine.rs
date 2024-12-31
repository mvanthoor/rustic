mod about;
mod comm_handler;
pub mod defs;
mod game_result;
mod main_loop;
mod search_handler;
mod utils;

use crate::{
    board::Board,
    comm::defs::{CommOut, CommType, IComm, Uci, XBoard},
    defs::EngineRunResult,
    engine::defs::{EngineOption, EngineOptionDefaults, EngineSetOption, EngineState},
    engine::defs::{ErrFatal, Information, Settings, UiElement},
    misc::{cmdline::CmdLine, perft},
    movegen::MoveGenerator,
    search::defs::{PerftData, SearchData, Verbosity, TT},
    search::{defs::SearchControl, Search},
};
use std::sync::{mpsc::Receiver, Arc, Mutex};

#[cfg(feature = "extra")]
use crate::{
    board::defs::Pieces,
    engine::defs::TexelSettings,
    extra::texel::defs::TunerLoadError,
    extra::texel::Tuner,
    extra::{testsuite, wizardry},
};

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

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    // Create e new engine.
    pub fn new() -> Self {
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

        // List of options that should be announced to the GUI.
        let options = vec![
            EngineOption::new(
                EngineSetOption::HASH,
                UiElement::Spin,
                Some(EngineOptionDefaults::HASH_DEFAULT.to_string()),
                Some(EngineOptionDefaults::HASH_MIN.to_string()),
                Some(EngineOptionDefaults::max_hash().to_string()),
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

        #[cfg(feature = "extra")]
        let texel = TexelSettings {
            file_name: cmdline.texel(),
        };

        // Create the engine itself.
        Self {
            quit: false,
            state: comm.info().startup_state(),
            settings: Settings {
                threads,
                verbosity,
                tt_size,

                #[cfg(feature = "extra")]
                texel,
            },
            options: Arc::new(options),
            cmdline,
            comm,
            board: Arc::new(Mutex::new(Board::new())),
            mg: Arc::new(MoveGenerator::new()),
            tt_perft,
            tt_search,
            info_rx: None,
            search: Search::new(tt_size),
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

        // Run perft if requested.
        if self.cmdline.perft() > 0 {
            perft::run(
                self.board.clone(),
                self.cmdline.perft(),
                Arc::clone(&self.mg),
                Arc::clone(&self.tt_perft),
                self.settings.tt_size > 0,
            );
            return Ok(());
        }

        // === Only available with "extra" features enabled. ===
        #[cfg(feature = "extra")]
        // Generate magic numbers if requested.
        if self.cmdline.has_wizardry() {
            wizardry::find_magics(Pieces::ROOK);
            wizardry::find_magics(Pieces::BISHOP);
            return Ok(());
        };

        #[cfg(feature = "extra")]
        // Run large EPD test suite if requested. Because the -p (perft)
        // option is not used in this scenario, the engine initializes the
        // search TT instead of the one for perft. The -e option is
        // not available in a non-extra compilation, so it cannot be
        // checked there. Just fix the issue by resizing both the perft and
        // search TT's appropriately for running the EPD suite.
        if self.cmdline.has_test() {
            self.tt_perft
                .lock()
                .expect(ErrFatal::LOCK)
                .resize(self.settings.tt_size);
            self.tt_search.lock().expect(ErrFatal::LOCK).resize(0);
            testsuite::run(Arc::clone(&self.tt_perft), self.settings.tt_size > 0);
            return Ok(());
        }

        #[cfg(feature = "extra")]
        if let Some(data_file) = self.settings.texel.file_name.to_owned() {
            let mut tuner = Tuner::new(data_file);

            match tuner.load() {
                Ok(()) => tuner.run(),
                Err(e) => match e {
                    TunerLoadError::DataFileReadError => println!("{e}"),
                },
            };

            return Ok(());
        }
        // =====================================================

        // In the main loop, the engine manages its resources so it will be
        // able to play legal chess and communicate with different user
        // interfaces.
        self.main_loop();

        // We're done and the engine exited without issues.
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
