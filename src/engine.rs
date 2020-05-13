use crate::{board::Board, extra::perft, movegen::MoveGenerator};

pub const ENGINE: &str = "Rustic";
pub const VERSION: &str = "Alpha 1";
pub const AUTHOR: &str = "Marcel Vanthoor";

pub struct Engine {
    move_generator: MoveGenerator,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            move_generator: MoveGenerator::new(),
        }
    }

    pub fn about(&self) {
        println!();
        println!("Engine: {} {}", ENGINE, VERSION);
        println!("Author: {}", AUTHOR);
    }

    pub fn perft(&mut self, board: &Board, depth: u8) {
        perft::run(&board, depth, &self.move_generator)
    }
}
