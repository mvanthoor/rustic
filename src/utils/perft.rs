use crate::board::make_move::make_move;
use crate::board::representation::Board;
use crate::board::unmake_move::unmake_move;
use crate::board::zobrist::ZobristRandoms;
use crate::movegen::movedefs::MoveListPool;
use crate::movegen::MoveGenerator;
use std::time::Instant;

#[allow(dead_code)]
pub fn bench(depth: u8) {
    let mut total_time: u128 = 0;
    let mut total_nodes: u64 = 0;
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();

    println!("Benchmarking perft 1-{} from starting position...", depth);

    for d in 1..=depth {
        let mut move_list_pool = MoveListPool::new();
        let mut perft_board: Board = Board::new(&zobrist_randoms, &move_generator, None);
        let now = Instant::now();
        let leaf_nodes = perft(&mut perft_board, d, &mut move_list_pool);
        let elapsed = now.elapsed().as_millis();
        let leaves_per_second = ((leaf_nodes * 1000) as f64 / elapsed as f64).floor();
        total_time += elapsed;
        total_nodes += leaf_nodes;
        println!(
            "Perft {}: {} ({} ms, {} leaves/sec)",
            d, leaf_nodes, elapsed, leaves_per_second
        );
    }

    let final_lnps = ((total_nodes * 1000) as f64 / total_time as f64).floor();
    println!("Total time spent: {} ms", total_time);
    println!("Execution speed: {} leaves/second", final_lnps);
}

#[allow(dead_code)]
pub fn perft(board: &mut Board, depth: u8, mlp: &mut MoveListPool) -> u64 {
    let mut leaf_nodes: u64 = 0;
    let index = depth as usize;

    if depth == 0 {
        return 1;
    }

    board
        .move_generator
        .gen_all_moves(&board, mlp.get_list_mut(index));
    for i in 0..mlp.get_list(index).len() {
        if !make_move(board, mlp.get_list(index).get_move(i)) {
            continue;
        };
        leaf_nodes += perft(board, depth - 1, mlp);
        unmake_move(board);
    }

    leaf_nodes
}
