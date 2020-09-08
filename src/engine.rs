use crate::{
    board::{defs::Pieces, Board},
    defs::{About, FEN_START_POSITION},
    extra::{perft, wizardry},
    interface,
    misc::cmdline,
    movegen::MoveGenerator,
};

pub struct Engine {
    cmdline_fen: String,
    cmdline_wizardry: bool,
    move_generator: MoveGenerator,
    board: Board,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            cmdline_fen: String::from(""),
            cmdline_wizardry: false,
            move_generator: MoveGenerator::new(),
            board: Board::new(),
        }
    }

    pub fn run(&mut self) {
        self.cmdline_get_values();
        self.about();
        interface::uci::run();

        println!("Engine running...");
        println!("FEN: {}", self.cmdline_fen);

        self.board.fen_read(Some(&self.cmdline_fen[..]));
        perft::run(&self.board, 6, &self.move_generator);

        if self.cmdline_wizardry {
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
        let cmdline = cmdline::get();
        self.cmdline_fen = cmdline
            .value_of("fen")
            .unwrap_or(FEN_START_POSITION)
            .to_string();
        self.cmdline_wizardry = cmdline.is_present("wizardry");
    }
}
