mod board;
mod defs;
mod extra;
mod movegenerator;
mod utils;

use board::representation::Board;
use defs::{AUTHOR, BLACK, ENGINE, VERSION, WHITE};
use extra::print;
use movegenerator::gen::{MoveList, MAX_LEGAL_MOVES};
use movegenerator::init::Movements;

/** Prints information about the engine to the screen. */
pub fn engine_info() {
    println!();
    println!("Engine: {} {}", ENGINE, VERSION);
    println!("Author: {}", AUTHOR);
}

fn main() {
    let mut board: Board = Default::default();
    let mut movements: Movements = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    let test_pos: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    board.initialize(test_pos);
    movements.initialize();

    engine_info();

    print::position(&board, None);

    movegenerator::gen::generate(&board, WHITE, &movements, &mut moves);
    print::movelist(&moves);
    println!("...");
    movegenerator::gen::generate(&board, BLACK, &movements, &mut moves);
    print::movelist(&moves);

    println!("Done.");
}
