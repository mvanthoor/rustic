mod board;
mod comm;
mod defs;
mod evaluation;
mod extra;
mod movegen;
mod utils;

use board::{fen::ERR_FEN_PARTS, representation::Board, zobrist::ZobristRandoms};
use comm::cli;
use extra::{perft, perftsuite, print};
use movegen::{movedefs::MoveList, MoveGenerator};
use std::env;

use utils::parse;

fn main() {
    let test_pos = Some("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();
    let mut board: Board = Board::new(&zobrist_randoms, &move_generator);
    let mut move_list: MoveList = MoveList::new();
    let setup_result = board.fen_read(None);
    let args: Vec<String> = env::args().collect();

    utils::engine_info();

    match setup_result {
        Ok(()) => {
            // print::position(&board, None);
            while cli::get_input(&mut board) != 0 {}
            // perft::run(&board, 6);
            // perftsuite::run_all_tests();
        }
        Err(e) => println!("Error in FEN-part: {}", ERR_FEN_PARTS[e as usize]),
    }
}
