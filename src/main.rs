mod board;
mod defs;
mod extra;
mod movegen;
mod utils;

use board::representation::Board;
use board::zobrist::ZobristRandoms;
use extra::print;
use movegen::movedefs::{MoveList, MAX_LEGAL_MOVES};
use movegen::MoveGenerator;
use utils::engine_info;

fn main() {
    let test_pos: &str = "r3k2r/2p3p1/8/3Ppp2/2n3Pp/1P6/P3PP2/R3K2R b KQkq g3 0 1";
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();
    let mut board: Board = Board::new(&zobrist_randoms, None);
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    engine_info();

    print::position(&board, None);

    move_generator.gen_all_moves(&board, &mut moves);
    print::movelist(&moves);

    println!("Done.");
}
