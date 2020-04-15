mod board;
mod comm;
mod defs;
mod evaluation;
mod extra;
mod movegen;
mod utils;

use board::{representation::Board, zobrist::ZobristRandoms};
use comm::cli;
use extra::{perft, perftsuite, print};
use movegen::{movedefs::MoveList, MoveGenerator};
use utils::parse;

fn main() {
    let test_pos: &str = "rnbqkbnr/pp1ppppp/2p5/8/PP6/8/2PPPPPP/RNBQKBNR w KQkq - 0 1";
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();
    let mut board: Board = Board::new(&zobrist_randoms, &move_generator, None);
    let mut move_list: MoveList = MoveList::new();

    utils::engine_info();
    print::position(&board, None);

    while cli::get_input(&mut board) != 0 {}
    // perft::run(&board, 7);
    // perftsuite::run_all_tests();
}
