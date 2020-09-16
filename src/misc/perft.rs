use crate::{
    board::Board,
    misc::print,
    movegen::{defs::MoveList, MoveGenerator},
};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

// This function runs perft(), while collecting speed information.
pub fn run(board: &Board, depth: u8, threads: u8, mg: Arc<MoveGenerator>) {
    let mut total_time: u128 = 0;
    let mut total_nodes: u64 = 0;

    println!("Benchmarking perft 1-{} on {} threads", depth, threads);

    print::position(board, None);

    // Perform all perfts for depths 1 up to and including "depth"
    for d in 1..=depth {
        // Everything has to be local for threading
        let mut perft_board: Board = board.clone();
        let arc_mg = mg.clone();

        // Current time
        let now = Instant::now();

        // let leaf_nodes = perft(&mut perft_board, d, &mg);

        // Run perft in thread
        let result = thread::spawn(move || perft(&mut perft_board, d, &arc_mg));
        let leaf_nodes = result.join().unwrap_or(0);

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
pub fn perft(board: &mut Board, depth: u8, mg: &MoveGenerator) -> u64 {
    let mut leaf_nodes: u64 = 0;
    let mut move_list: MoveList = MoveList::new();

    if depth == 0 {
        return 1;
    }

    mg.gen_all_moves(board, &mut move_list);
    let nr_of_moves = move_list.len();

    for i in 0..nr_of_moves {
        let m = move_list.get_move(i);
        let legal = board.make(m, mg);

        if legal {
            leaf_nodes += perft(board, depth - 1, mg);
            board.unmake();
        }
    }

    leaf_nodes
}
