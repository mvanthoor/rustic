mod about;
mod comm_handler;
pub mod defs;
mod game_result;
mod main_loop;
mod search_handler;
mod utils;

use crate::{
    basetypes::error::ErrFatal,
    board::Board,
    comm::defs::{
        CommOption, CommOut, CommType, EngineState, IComm, Information, Uci, UiElement, XBoard,
    },
    defs::EngineRunResult,
    engine::defs::{EngineOptionDefaults, EngineSetOption, Settings},
    misc::{cmdline::CmdLine, perft},
    movegen::MoveGenerator,
    search::{
        defs::{SearchControl, Verbosity},
        Search,
    },
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
    options: Arc<Vec<CommOption>>,          // Engine options exported to the GUI.
    cmdline: CmdLine,                       // Command line interpreter.
    comm: Box<dyn IComm>,                   // UCI/XBoard communication (active).
    board: Arc<Mutex<Board>>,               // This is the main engine board.
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
            CommOption::new(
                EngineSetOption::HASH,
                UiElement::Spin,
                Some(EngineOptionDefaults::HASH_DEFAULT.to_string()),
                Some(EngineOptionDefaults::HASH_MIN.to_string()),
                Some(EngineOptionDefaults::max_hash().to_string()),
            ),
            CommOption::new(
                EngineSetOption::CLEAR_HASH,
                UiElement::Button,
                None,
                None,
                None,
            ),
        ];

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
        let position = self.determine_startup_position();
        self.board
            .lock()
            .expect(ErrFatal::LOCK)
            .fen_setup(Some(&position))?;

        // Run perft if requested.
        if self.cmdline.perft() > 0 {
            perft::run(
                &position,
                self.cmdline.perft(),
                Arc::clone(&self.mg),
                self.settings.tt_size,
            )?;
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
        if self.cmdline.has_test() {
            testsuite::run(self.settings.tt_size);
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
