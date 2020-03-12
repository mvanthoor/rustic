mod board;
mod defs;
mod extra;
mod movegenerator;
mod utils;

use board::representation::Board;
use defs::{BLACK, WHITE};
use extra::print;
use movegenerator::movedefs::{MoveList, MAX_LEGAL_MOVES};
use movegenerator::MoveGenerator;
use utils::engine_info;

fn main() {
    let mut board: Board = Default::default();
    let mut move_generator: MoveGenerator = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    let test_pos: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    board.initialize(test_pos);
    move_generator.initialize();

    engine_info();

    print::position(&board, None);

    move_generator.gen_all_moves(&board, WHITE, &mut moves);
    print::movelist(&moves);
    println!("...");
    move_generator.gen_all_moves(&board, BLACK, &mut moves);
    print::movelist(&moves);

    println!("Done.");
}
