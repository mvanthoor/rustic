use crate::{
    board::Board, defs::FEN_START_POSITION, interface, misc::cmdline, movegen::MoveGenerator,
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

    pub fn cmdline_get_values(&mut self) {
        let cmdline = cmdline::get();
        self.cmdline_fen = cmdline
            .value_of("fen")
            .unwrap_or(FEN_START_POSITION)
            .to_string();
    }

    pub fn run(&mut self) {
        self.cmdline_get_values();
        println!("Engine running...");
        interface::uci::run();
        println!("FEN: {}", self.cmdline_fen);
    }
}
