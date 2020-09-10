use crate::{
    board::{defs::Pieces, Board},
    comm,
    defs::{About, EngineRunResult},
    extra::{perft, wizardry},
    misc::cmdline::CmdLine,
    movegen::MoveGenerator,
};

// This struct holds the chess engine and its functions. The reason why
// this is not done in the main program, is because this struct can contain
// member functions and allows functions to be associated with it. This
// setup makes it possible to create an actual engine with parts, such as a
// Command Line, a Move Generator, a Board, and so on, instead of having
// all those parts in the global space.
pub struct Engine {
    cmdline: CmdLine,
    move_generator: MoveGenerator,
    board: Board,
}

impl Engine {
    // Create e new engine.
    pub fn new() -> Self {
        Self {
            cmdline: CmdLine::new(),
            move_generator: MoveGenerator::new(),
            board: Board::new(),
        }
    }

    // Run the engine.
    pub fn run(&mut self) -> EngineRunResult {
        // Print engine information
        self.about();

        // Setup according to provided FEN-string, if any.
        let fen = &self.cmdline.fen()[..];
        self.board.fen_read(Some(fen))?;

        // Run a specific action if requested, or start the engine.
        let mut action_requested = false;

        // Run perft if requested.
        if self.cmdline.perft() > 0 {
            action_requested = true;
            println!("FEN: {}", fen);
            perft::run(&self.board, self.cmdline.perft(), &self.move_generator);
        }

        // Generate magic numbers if requested.
        if self.cmdline.wizardry() {
            action_requested = true;
            wizardry::find_magics(Pieces::ROOK);
            wizardry::find_magics(Pieces::BISHOP);
        };

        // Start the engine, if no other actions requested.
        if !action_requested {
            match &self.cmdline.comm()[..] {
                "uci" => comm::uci::get_input(),
                "xboard" => comm::xboard::get_input(),
                "console" => comm::console::get_input(),
                _ => (),
            }
        };

        // Engine exits correctly.
        Ok(())
    }

    // Print information about the engine.
    fn about(&self) {
        println!();
        println!("Engine: {} {}", About::ENGINE, About::VERSION);
        println!("Author: {} <{}>", About::AUTHOR, About::EMAIL);
        println!("Description: {}", About::DESCRIPTION);
    }
}
