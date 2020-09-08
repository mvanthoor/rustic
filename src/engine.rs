use crate::{
    board::Board,
    defs::{About, FEN_START_POSITION},
    extra::perft,
    interface,
    misc::cmdline,
    movegen::MoveGenerator,
};

pub struct Engine {
    cmdline_fen: String,
    move_generator: MoveGenerator,
    board: Board,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            cmdline_fen: String::from(""),
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
    }
}
