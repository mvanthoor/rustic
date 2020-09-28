mod main_loop;
mod utils;

use crate::{
    board::Board,
    comm::{console::Console, CommReport, CommType, IComm},
    defs::EngineRunResult,
    misc::{cmdline::CmdLine, perft},
    movegen::MoveGenerator,
    search::{Search, SearchReport},
};
use std::sync::{Arc, Mutex};

#[cfg(feature = "extra")]
use crate::{
    board::defs::Pieces,
    extra::{testsuite, wizardry},
};

// This struct holds messages that are reported on fatal engine errors.
// These should never happen; if they do the engine is in an unknown state,
// and it will panic without trying any recovery whatsoever.
pub struct ErrFatal;
impl ErrFatal {
    pub const CREATE_COMM: &'static str = "Comm creation failed.";
    pub const LOCK: &'static str = "Lock failed.";
    pub const READ_IO: &'static str = "Reading I/O failed.";
    pub const FLUSH_IO: &'static str = "Flushing I/O failed.";
    pub const HANDLE: &'static str = "Broken handle.";
    pub const THREAD: &'static str = "Thread has failed.";
    pub const CHANNEL: &'static str = "Broken channel.";
}

// This struct holds the engine's settings.
pub struct Settings {
    threads: usize,
}

// This enum provides informatin to the engine, with regard to incoming
// messages and search results.
#[derive(PartialEq)]
pub enum Information {
    Comm(CommReport),
    Search(SearchReport),
}

// This struct holds the chess engine and its functions, so they are not
// all seperate entities in the global space.
pub struct Engine {
    quit: bool,               // Flag that will quit the main thread.
    settings: Settings,       // Struct holding all the settings.
    cmdline: CmdLine,         // Command line interpreter.
    comm: Box<dyn IComm>,     // Communications (active).
    board: Arc<Mutex<Board>>, // Board.
    mg: Arc<MoveGenerator>,   // Move Generator.
    search: Search,           // Search (active).
}

impl Engine {
    // Create e new engine.
    pub fn new() -> Self {
        // Create the command-line object.
        let c = CmdLine::new();

        // Create the communication interface
        let i: Box<dyn IComm> = match &c.comm()[..] {
            // CommType::UCI => Box::new(Uci::new()),
            // CommType::XBOARD => Box::new(Xboard::new()),
            CommType::CONSOLE => Box::new(Console::new()),
            _ => panic!(ErrFatal::CREATE_COMM),
        };

        let t = c.threads();

        // Create the engine itself.
        Self {
            quit: false,
            settings: Settings { threads: t },
            cmdline: c,
            comm: i,
            board: Arc::new(Mutex::new(Board::new())),
            mg: Arc::new(MoveGenerator::new()),
            search: Search::new(),
        }
    }

    // Run the engine.
    pub fn run(&mut self) -> EngineRunResult {
        self.about(); // Print engine information.
        self.setup_position()?; // ? means: Abort if position setup fails.

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

        println!("Engine shutdown completed.");

        // Engine exits correctly. There are only two other possibilities
        // to exit: if FEN-setup fails (which is reported and the engine
        // exits normally through the main() function), and with a fatal
        // error which will make the engine crash. In both cases, it'll not
        // reach the Ok(()) here.
        Ok(())
    }
}
