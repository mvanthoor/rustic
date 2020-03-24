use crate::board::make_move::make_move;
use crate::board::representation::Board;
use crate::board::unmake_move::unmake_move;
use crate::movegen::movedefs::{MoveList, MoveListPool};
use crate::movegen::MoveGenerator;
use std::time::Instant;

pub fn run(board: &Board, depth: u8, mg: &MoveGenerator) {
    for d in 1..=depth {
        let mut perft_board = board.clone();
        let mut move_list_pool = MoveListPool::new();
        let now = Instant::now();
        let leaf_nodes = perft(&mut perft_board, d, &mut move_list_pool, &mg);
        let elapsed = now.elapsed().as_millis();
        let moves_ps = ((leaf_nodes * 1000) as f64 / elapsed as f64).floor();
        println!(
            "Peft {}: {} ({} ms, {} moves/sec)",
            d, leaf_nodes, elapsed, moves_ps
        );
    }
}

fn perft(board: &mut Board, depth: u8, mlp: &mut MoveListPool, mg: &MoveGenerator) -> u64 {
    let mut leaf_nodes: u64 = 0;
    let current = depth as usize;

    if depth == 0 {
        return 1;
    }

    mg.gen_all_moves(&board, &mut mlp.lists[current]);
    for i in 0..mlp.lists[current].len() {
        if !make_move(board, mlp.lists[current].get(i), mg) {
            continue;
        };
        leaf_nodes += perft(board, depth - 1, mlp, mg);
        unmake_move(board);
    }

    leaf_nodes
}
