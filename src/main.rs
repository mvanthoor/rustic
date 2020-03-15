mod board;
mod defs;
mod extra;
mod movegenerator;
mod utils;

use board::representation::Board;
use board::zobrist;
use extra::print;
use movegenerator::movedefs::{MoveList, MAX_LEGAL_MOVES};
use movegenerator::MoveGenerator;
use utils::engine_info;

fn main() {
    let mut board: Board = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);
    let move_generator: MoveGenerator = Default::default();

    let test_pos: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    board.setup_fen(test_pos);

    engine_info();

    print::position(&board, None);

    move_generator.gen_all_moves(&board, &mut moves);
    print::movelist(&moves);

    zobrist::initialize();

    println!("Done.");
}
