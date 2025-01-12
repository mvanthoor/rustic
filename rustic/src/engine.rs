mod about;
mod cmdline;
mod defs;
mod game_result;
mod handlers;
mod main_loop;
mod states;
mod uci_options;
mod utils;

use crate::engine::{
    about::{AUTHOR, ENGINE, VERSION},
    cmdline::CmdLine,
    defs::Settings,
};
use librustic::{
    basetypes::error::ErrFatal,
    board::Board,
    communication::{
        defs::{EngineState, IComm, Information},
        feature::Feature,
        uci::{cmd_out::UciOut, Uci},
    },
    defs::{About, EngineRunResult},
    misc::perft,
    movegen::MoveGenerator,
    search::{
        defs::{SearchControl, Verbosity},
        Search,
    },
};
use std::sync::{mpsc::Receiver, Arc, Mutex};

// This struct holds the chess engine and its functions, so they are not
// all separate entities in the global space.
pub struct Engine {
    quit: bool,                             // Flag that will quit the main thread.
    debug: bool,                            // Send errors/debug info to GUI
    state: EngineState,                     // Keeps the current engine activity.
    settings: Settings,                     // Struct holding all the settings.
    features: Arc<Vec<Feature>>,            // Engine options exported to the GUI.
    cmdline: CmdLine,                       // Command line interpreter.
    comm: Box<dyn IComm>,                   // UCI/XBoard communication (active).
    board: Arc<Mutex<Board>>,               // This is the main engine board.
    mg: Arc<MoveGenerator>,                 // Move Generator.
    info_rx: Option<Receiver<Information>>, // Receiver for incoming information.

    search: Search, // Search object (active).
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

        let comm: Box<dyn IComm> = Box::new(Uci::new(about));

        // Get engine settings from the command-line.
        let threads = cmdline.threads();
        let verbosity = if cmdline.has_quiet() {
            Verbosity::Quiet
        } else {
            Verbosity::Full
        };
        let tt_size = cmdline.hash();

        // These are features the engine supports. It sends them to the
        // communication module so they will be announced to the GUI.
        let features = vec![uci_options::hash::new(), uci_options::clear_hash::new()];

        // Create the engine itself.
        Self {
            quit: false,
            debug: cmdline.has_debug(),
            state: EngineState::UciNotUsed,
            settings: Settings {
                threads,
                verbosity,
                tt_size,
            },
            features: Arc::new(features),
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
        // Required for Stateful protocols such as XBoard.
        self.state = self.comm.properties().startup_state();

        // The UCI GUI's I tested don't seem to care about non-uci input
        // over multiple lines. Some XBoard user interfaces do choke on
        // this. Use a simpler, online-about instead.
        if self.comm.properties().support_fancy_about() {
            self.print_fancy_about(&self.settings, self.comm.properties().protocol_name());
        } else {
            self.print_simple_about(&self.settings, self.comm.properties().protocol_name());
        }

        // Setup position and abort if this fails.
        let position = self.determine_startup_position();
        self.board
            .lock()
            .expect(ErrFatal::LOCK)
            .fen_setup(Some(&position))?;

        // Run perft if requested, then quit.
        if self.cmdline.perft() > 0 {
            perft::run(
                &position,
                self.cmdline.perft(),
                Arc::clone(&self.mg),
                self.settings.tt_size,
            )?;
            return Ok(());
        }

        // Finally start the actual engine.
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
}
