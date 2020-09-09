use crate::{
    board::{defs::Pieces, Board},
    comm,
    defs::About,
    extra::{perft, wizardry},
    misc::cmdline::CmdLine,
    movegen::MoveGenerator,
};

pub struct Engine {
    cmdline: CmdLine,
    move_generator: MoveGenerator,
    board: Board,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            cmdline: CmdLine::new(),
            move_generator: MoveGenerator::new(),
            board: Board::new(),
        }
    }

    pub fn run(&mut self) {
        let fen = &self.cmdline.fen()[..];
        let depth = self.cmdline.perft();

        self.about();
        match &self.cmdline.comm()[..] {
            "uci" => comm::uci::get_input(),
            "xboard" => comm::xboard::get_input(),
            "console" => comm::console::get_input(),
            _ => (),
        }

        println!("Engine running...");
        println!("FEN: {}", self.cmdline.fen());

        self.board.fen_read(Some(fen));
        perft::run(&self.board, depth, &self.move_generator);

        if self.cmdline.wizardry() {
            wizardry::find_magics(Pieces::ROOK);
            wizardry::find_magics(Pieces::BISHOP);
        }
    }

    fn about(&self) {
        println!();
        println!("Engine: {} {}", About::ENGINE, About::VERSION);
        println!("Author: {} <{}>", About::AUTHOR, About::EMAIL);
        println!("Description: {}", About::DESCRIPTION);
    }
}
