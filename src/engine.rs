mod main_loop;
mod utils;

use crate::{
    board::Board,
    comm::{console::Console, CommControl, CommReport, CommType, IComm},
    defs::EngineRunResult,
    misc::{cmdline::CmdLine, perft},
    movegen::MoveGenerator,
    search::{Search, SearchControl, SearchReport},
};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

#[cfg(feature = "extra")]
use crate::{
    board::defs::Pieces,
    extra::{testsuite, wizardry},
};

// If one of these errors happens, there is a fatal situation within the
// engine or one of its threads, and it will crash.
struct ErrFatal {}
impl ErrFatal {
    const COMM_CREATION: &'static str = "Comm creation failed.";
    const BOARD_LOCK: &'static str = "Board lock failed.";
    const CHANNEL_BROKEN: &'static str = "Channel is broken.";
}

// This struct holds the engine's settings.
pub struct Settings {
    threads: u8,
}

// This enum provides informatin to the engine, with regard to incoming
// messages and search results.
pub enum Information {
    Comm(CommReport),
    Search(SearchReport),
}

// This stores sender parts of control channels.
pub struct ControlTx {
    comm: Option<Sender<CommControl>>,
    search: Option<Sender<SearchControl>>,
}

impl ControlTx {
    pub fn new(c: Option<Sender<CommControl>>, s: Option<Sender<SearchControl>>) -> Self {
        Self { comm: c, search: s }
    }
}

// This struct holds the chess engine and its functions. The reason why
// this is not done in the main program, is because this struct can contain
// member functions and other structs, so these don't have to be in the
// global space.
pub struct Engine {
    running: bool,
    settings: Settings,
    cmdline: CmdLine,
    comm: Box<dyn IComm>,
    board: Arc<Mutex<Board>>,
    mg: Arc<MoveGenerator>,
    search: Search,
    ctrl_tx: ControlTx,
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
            _ => panic!(ErrFatal::COMM_CREATION),
        };

        // Create the engine itself.
        Self {
            running: true,
            settings: Settings {
                threads: c.threads(),
            },
            cmdline: c,
            comm: i,
            board: Arc::new(Mutex::new(Board::new())),
            mg: Arc::new(MoveGenerator::new()),
            search: Search::new(),
            ctrl_tx: ControlTx::new(None, None),
        }
    }

    // Run the engine.
    pub fn run(&mut self) -> EngineRunResult {
        self.about();
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

        // Engine exits correctly.
        Ok(())
    }

    pub fn comm_tx(&self, message: CommControl) {
        if let Some(tx) = &self.ctrl_tx.comm {
            tx.send(message).expect(ErrFatal::CHANNEL_BROKEN);
        }
    }

    pub fn search_tx(&self, message: SearchControl) {
        if let Some(tx) = &self.ctrl_tx.search {
            tx.send(message).expect(ErrFatal::CHANNEL_BROKEN);
        }
    }
}
