use crate::{
    board::Board,
    comm::{console::Console, IComm},
    defs::{About, EngineRunResult},
    misc::{cmdline::CmdLine, perft},
    movegen::MoveGenerator,
};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[cfg(feature = "extra")]
use crate::{
    board::defs::Pieces,
    extra::{testsuite, wizardry},
};

pub struct Settings {
    threads: u8,
}

// This struct holds the chess engine and its functions. The reason why
// this is not done in the main program, is because this struct can contain
// member functions and allows functions to be associated with it. This
// setup makes it possible to create an actual engine with parts, such as a
// Command Line, a Move Generator, a Board, and so on, instead of having
// all those parts in the global space.
pub struct Engine {
    settings: Settings,
    cmdline: CmdLine,
    comm: Box<dyn IComm>,
    mg: Arc<MoveGenerator>,
    board: Arc<Mutex<Board>>,
    comm_handle: Option<JoinHandle<()>>,
}

impl Engine {
    // Create e new engine.
    pub fn new() -> Self {
        // Create the command-line object.
        let c = CmdLine::new();

        // Create the communication interface
        let i: Box<dyn IComm> = match &c.comm()[..] {
            // "uci" => Box::new(Uci::new()),
            // "xboard" => Box::new(Xboard::new()),
            "console" => Box::new(Console::new()),
            _ => panic!("Engine communication interface failed."),
        };

        Self {
            settings: Settings {
                threads: c.threads(),
            },
            cmdline: c,
            comm: i,
            mg: Arc::new(MoveGenerator::new()),
            board: Arc::new(Mutex::new(Board::new())),
            comm_handle: None,
        }
    }

    // Run the engine.
    pub fn run(&mut self) -> EngineRunResult {
        // Print engine information
        self.about();

        // Setup according to provided FEN-string, if any.
        let fen = &self.cmdline.fen()[..];

        // Abort if position setup fails due to invalid FEN.
        self.board.lock().unwrap().fen_read(Some(fen))?;

        // Run a specific action if requested, or start the engine.
        let mut action_requested = false;

        // Run perft if requested.
        if self.cmdline.perft() > 0 {
            action_requested = true;
            println!("FEN: {}", fen);
            perft::run(
                self.board.clone(),
                self.cmdline.perft(),
                self.settings.threads,
                self.mg.clone(),
            );
        }

        // === Only available with "extra" features enabled. ===
        #[cfg(feature = "extra")]
        // Generate magic numbers if requested.
        if self.cmdline.wizardry() {
            action_requested = true;
            wizardry::find_magics(Pieces::ROOK);
            wizardry::find_magics(Pieces::BISHOP);
        };

        #[cfg(feature = "extra")]
        // Run large EPD test suite if requested.
        if self.cmdline.test() {
            action_requested = true;
            testsuite::run();
        }
        // =====================================================

        // Start the communication thread if no other actions requested.
        if !action_requested {
            self.comm_handle = Some(self.comm.start(self.board.clone()));
        }

        // Wait for the communication thread to finish.
        if let Some(h) = self.comm_handle.take() {
            h.join().expect("Closing communication failed.");
        }

        // Engine exits correctly.
        Ok(())
    }

    // Print information about the engine.
    fn about(&self) {
        println!("Program: {} {}", About::ENGINE, About::VERSION);
        println!("Author: {} <{}>", About::AUTHOR, About::EMAIL);
        println!("Description: {}", About::DESCRIPTION);

        #[cfg(debug_assertions)]
        println!("Notice: Running in debug mode");
    }
}
