mod main_loop;
mod misc;

use crate::{
    board::Board,
    comm::{console::Console, CommType, IComm},
    defs::{EngineRunResult, FEN_KIWIPETE_POSITION},
    misc::{cmdline::CmdLine, perft},
    movegen::MoveGenerator,
    search::Search,
};
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
    const FAIL_QUIT_SEARCH: &'static str = "Stopping search failed.";
}

// This struct holds the engine's settings.
pub struct Settings {
    threads: u8,
}

// This struct holds the chess engine and its functions. The reason why
// this is not done in the main program, is because this struct can contain
// member functions and other structs, so these don't have to be in the
// global space.
pub struct Engine {
    settings: Settings,
    cmdline: CmdLine,
    comm: Box<dyn IComm>,
    mg: Arc<MoveGenerator>,
    board: Arc<Mutex<Board>>,
    search: Search,
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
            settings: Settings {
                threads: c.threads(),
            },
            cmdline: c,
            comm: i,
            mg: Arc::new(MoveGenerator::new()),
            board: Arc::new(Mutex::new(Board::new())),
            search: Search::new(),
        }
    }

    // Run the engine.
    pub fn run(&mut self) -> EngineRunResult {
        // Print engine information.
        self.about();

        // Get either the provided FEN-string or KiwiPete. If both are
        // provided, the KiwiPete position takes precedence.
        let f = &self.cmdline.fen()[..];
        let kp = self.cmdline.has_kiwipete();
        let fen = if kp { FEN_KIWIPETE_POSITION } else { f };

        // Lock the board, setup the FEN-string, and drop the lock.
        let mut mtx_board = self.board.lock().expect(ErrFatal::BOARD_LOCK);
        mtx_board.fen_read(Some(fen))?;
        std::mem::drop(mtx_board);

        // Run a specific action if requested, or start the engine.
        let mut action_requested = false;

        // Run perft if requested.
        if self.cmdline.perft() > 0 {
            action_requested = true;
            println!("FEN: {}", fen);
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

        // Start the engine if no other actions are requested.
        if !action_requested {
            // Start the main engine loop.
            self.main_loop();
        }

        // Engine exits correctly.
        Ok(())
    }
}
