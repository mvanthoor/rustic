use crate::{
    board::{defs::Pieces, Board},
    comm,
    defs::{About, FEN_START_POSITION},
    extra::{perft, wizardry},
    misc::cmdline,
    movegen::MoveGenerator,
};

struct CmdLine {
    communication: String,
    fen: String,
    perft: u8,
    wizardry: bool,
}

pub struct Engine {
    cmdline: CmdLine,
    move_generator: MoveGenerator,
    board: Board,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            cmdline: CmdLine {
                communication: String::from(""),
                fen: String::from(""),
                perft: 0,
                wizardry: false,
            },
            move_generator: MoveGenerator::new(),
            board: Board::new(),
        }
    }

    pub fn run(&mut self) {
        self.cmdline_get_values();
        self.about();
        comm::uci::run();

        println!("Engine running...");
        println!("FEN: {}", self.cmdline.fen);

        self.board.fen_read(Some(&self.cmdline.fen[..]));
        perft::run(&self.board, 6, &self.move_generator);

        if self.cmdline.wizardry {
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

    fn cmdline_get_values(&mut self) {
        let c = cmdline::get();
        self.cmdline.communication = c.value_of("communication").unwrap_or("uci").to_string();
        self.cmdline.fen = c.value_of("fen").unwrap_or(FEN_START_POSITION).to_string();
        self.cmdline.wizardry = c.is_present("wizardry");
    }
}
