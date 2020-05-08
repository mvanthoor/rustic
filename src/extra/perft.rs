use crate::{
    board::{playmove, representation::Board},
    extra::print,
    movegen::movelist::MoveList,
};
use std::time::Instant;

// This function runs perft(), while collecting speed information.
#[allow(dead_code)]
pub fn run(board: &Board, depth: u8) {
    let mut total_time: u128 = 0;
    let mut total_nodes: u64 = 0;

    println!("Benchmarking perft 1-{}: ", depth);
    print::position(board, None);

    // Perform all perfts for depths 1 up to and including "depth"
    for d in 1..=depth {
        let mut perft_board: Board = board.clone();
        let now = Instant::now();
        let leaf_nodes = perft(&mut perft_board, d);
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

// This is the actual Perft function.
#[allow(dead_code)]
pub fn perft(board: &mut Board, depth: u8) -> u64 {
    let mut leaf_nodes: u64 = 0;
    let mut move_list: MoveList = MoveList::new();

    if depth == 0 {
        return 1;
    }

    board.gen_all_moves(&mut move_list);
    let nr_of_moves = move_list.len();

    for i in 0..nr_of_moves {
        let m = move_list.get_move(i);
        let legal = playmove::make(board, m);

        if legal {
            leaf_nodes += perft(board, depth - 1);
            playmove::unmake(board);
        }
    }

    leaf_nodes
}
