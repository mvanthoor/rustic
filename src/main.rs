mod board;
mod defs;
mod extra;
mod movegen;
mod utils;

use board::make_move::make_move;
use board::representation::Board;
use board::unmake_move::unmake_move;
use board::zobrist::ZobristRandoms;
use extra::print;
use movegen::movedefs::{MoveList, MAX_LEGAL_MOVES};
use movegen::MoveGenerator;
use utils::engine_info;

fn perft(board: &mut Board, depth: u8, mg: &MoveGenerator) -> u64 {
    let mut move_list: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);
    let mut nodes: u64 = 0;

    if depth == 0 {
        return 1;
    }

    mg.gen_all_moves(&board, &mut move_list);
    for m in move_list {
        if !make_move(board, m, mg) {
            continue;
        }
        nodes += perft(board, depth - 1, mg);
        unmake_move(board);
    }

    nodes
}

fn main() {
    let test_pos: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();
    let mut board: Board = Board::new(&zobrist_randoms, Some(test_pos));
    // let mut move_list: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    engine_info();
    print::position(&board, None);

    const TEST: u8 = 5;
    for i in 1..=TEST {
        let mut perft_board: Board = Board::new(&zobrist_randoms, Some(test_pos));
        let x = perft(&mut perft_board, i, &move_generator);
        println!("Peft {}: {}", i, x);
    }

    println!("Finished.");
}
