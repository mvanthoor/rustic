use crate::board::make_move::make_move;
use crate::board::representation::Board;
use crate::board::unmake_move::unmake_move;
use crate::movegen::movedefs::{MoveList, MAX_LEGAL_MOVES};
use crate::movegen::MoveGenerator;

pub fn run(board: &Board, depth: u8, mg: &MoveGenerator) {
    for i in 1..=depth {
        let mut perft_board = board.clone();
        let x = perft(&mut perft_board, i, &mg);
        println!("Peft {}: {}", i, x);
    }
}

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
