use crate::{
    board::{defs::FenSetupResult, Board},
    extra::perft,
    movegen::MoveGenerator,
};

pub const ENGINE: &str = "Rustic";
pub const VERSION: &str = "Alpha 1";
pub const AUTHOR: &str = "Marcel Vanthoor";

pub struct Engine {
    board: Board,
    move_generator: MoveGenerator,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            move_generator: MoveGenerator::new(),
        }
    }

    pub fn about(&self) {
        println!();
        println!("Engine: {} {}", ENGINE, VERSION);
        println!("Author: {}", AUTHOR);
    }

    pub fn perft(&self, depth: u8) {
        perft::run(&self.board, depth, &self.move_generator)
    }

    pub fn position_setup(&mut self, fen: Option<&str>) -> FenSetupResult {
        self.board.fen_read(fen)
    }
}
