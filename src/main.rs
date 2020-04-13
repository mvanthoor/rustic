mod board;
mod comm;
mod defs;
mod evaluation;
mod extra;
mod movegen;
mod utils;

use board::{representation::Board, zobrist::ZobristRandoms};
use comm::cli;
use extra::perft;
use extra::perftsuite;
use extra::print;
use movegen::movedefs::MoveList;
use movegen::MoveGenerator;

fn main() {
    let test_pos: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
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
