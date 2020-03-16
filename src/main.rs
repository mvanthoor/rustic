mod board;
mod defs;
mod extra;
mod movegen;
mod utils;

use board::representation::Board;
use board::zobrist;
use extra::print;
use movegen::movedefs::{MoveList, MAX_LEGAL_MOVES};
use movegen::MoveGenerator;
use utils::engine_info;

fn main() {
    let test_pos: &str = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
    let mut board: Board = Board::new(Some(test_pos));
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);
    let move_generator: MoveGenerator = MoveGenerator::new();

    engine_info();

    print::position(&board, None);

    move_generator.gen_all_moves(&board, &mut moves);
    print::movelist(&moves);

    zobrist::initialize();

    println!("Done.");
}
