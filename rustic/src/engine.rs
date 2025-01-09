mod about;
mod cmdline;
mod comm_handler;
pub mod defs;
mod game_result;
mod main_loop;
mod search_handler;
mod utils;

use crate::engine::{
    about::{AUTHOR, ENGINE, VERSION},
    cmdline::CmdLine,
    defs::Settings,
};
use librustic::{
    basetypes::error::ErrFatal,
    board::Board,
    comm::defs::{EngineOptionDefaults, EngineSetOption, EngineState, Information},
    communication::{
        defs::{Features, IComm, UiElement},
        uci::{cmd_in::UciIn, cmd_out::UciOut, Uci},
    },
    defs::{About, EngineRunResult},
    misc::perft,
    movegen::MoveGenerator,
    search::{
        defs::{SearchControl, SearchReport, Verbosity},
        Search,
    },
};
use std::sync::{mpsc::Receiver, Arc, Mutex};

// This struct holds the chess engine and its functions, so they are not
// all separate entities in the global space.
pub struct Engine {
    quit: bool,                                // Flag that will quit the main thread.
    state: EngineState,                        // Keeps the current engine activity.
    settings: Settings,                        // Struct holding all the settings.
    options: Arc<Vec<Features>>,               // Engine options exported to the GUI.
    cmdline: CmdLine,                          // Command line interpreter.
    comm: Box<dyn IComm>,                      // UCI/XBoard communication (active).
    board: Arc<Mutex<Board>>,                  // This is the main engine board.
    mg: Arc<MoveGenerator>,                    // Move Generator.
    info_rx: Option<Receiver<UciIn>>,          // Receiver for incoming information.
    search_rx: Option<Receiver<SearchReport>>, // Search report receiver
    search: Search,                            // Search object (active).
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
        let about = About::new(
            String::from(ENGINE),
            String::from(VERSION),
            String::from(AUTHOR),
        );

        // Create the communication interface
        // let comm: Box<dyn IComm> = match cmdline.comm().as_str() {
        //     CommType::XBOARD => Box::new(XBoard::new(about)),
        //     CommType::UCI => Box::new(Uci::new(about)),
        //     _ => panic!("{}", ErrFatal::CREATE_COMM),
        // };

        let comm: Box<dyn IComm> = Box::new(Uci::new(about));

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
            Features::new(
                EngineSetOption::HASH,
                UiElement::Spin,
                Some(EngineOptionDefaults::HASH_DEFAULT.to_string()),
                Some(EngineOptionDefaults::HASH_MIN.to_string()),
                Some(EngineOptionDefaults::max_hash().to_string()),
            ),
            Features::new(
                EngineSetOption::CLEAR_HASH,
                UiElement::Button,
                None,
                None,
                None,
            ),
        ];

        // Create the engine itself.
        Self {
            quit: false,
            state: EngineState::Waiting,
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
            info_rx: None,
            search_rx: None,
            search: Search::new(tt_size),
        }
    }

    // Run the engine.
    pub fn run(&mut self) -> EngineRunResult {
        // if self.comm.info().supports_fancy_about() {
        //     self.print_fancy_about(&self.settings, self.comm.info().protocol_name());
        // } else {
        //     self.print_simple_about(&self.settings, self.comm.info().protocol_name());
        // }

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
        self.comm.send(UciOut::Quit);
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
