use crate::{
    board::Board,
    misc::print,
    movegen::{defs::MoveList, MoveGenerator},
};
use std::{sync::Arc, thread, time::Instant};

// This function runs perft(), while collecting speed information.
// It uses iterative deepening, so when running perft(7), it will output
// the results of perft(1) up to and including perft(7).
pub fn run(board: &Board, depth: u8, threads: u8, mg: Arc<MoveGenerator>) {
    let mut total_time: u128 = 0;
    let mut total_nodes: u64 = 0;

    println!("Benchmarking perft 1-{} on {} threads", depth, threads);

    print::position(board, None);

    // Perform all perfts for depths 1 up to and including "depth"
    for d in 1..=depth {
        // Everything has to be local for threading
        let mut perft_board: Board = board.clone();

        // Current time
        let now = Instant::now();
        let mut leaf_nodes = 0;

        // Run in the main thread if only one thread is requested.
        if threads == 1 {
            leaf_nodes = perft(&mut perft_board, d, &mg);
        }

        // In case of more than one thread, split up the work.
        // (Preliminary: for now, it only spawns one thread.)
        if threads >= 2 {
            let arc_mg = mg.clone();
            let result = thread::spawn(move || perft(&mut perft_board, d, &arc_mg));
            leaf_nodes = result.join().unwrap_or(0);
        }

        // Measure time and speed
        let elapsed = now.elapsed().as_millis();
        let leaves_per_second = ((leaf_nodes * 1000) as f64 / elapsed as f64).floor();

        // Add tot totals for final calculation at the very end.
        total_time += elapsed;
        total_nodes += leaf_nodes;

        println!(
            "Perft {}: {} ({} ms, {} leaves/sec)",
            d, leaf_nodes, elapsed, leaves_per_second
        );
    }

    // Final calculation of the entire time taken, and average speed of leaves/second.
    let final_lnps = ((total_nodes * 1000) as f64 / total_time as f64).floor();
    println!("Total time spent: {} ms", total_time);
    println!("Execution speed: {} leaves/second", final_lnps);
}

// This is the actual Perft function.
pub fn perft(board: &mut Board, depth: u8, mg: &MoveGenerator) -> u64 {
    let mut leaf_nodes: u64 = 0;
    let mut move_list: MoveList = MoveList::new();

    // Count each visited leaf node.
    if depth == 0 {
        return 1;
    }

    // Generate all moves in the position
    mg.gen_all_moves(board, &mut move_list);
    let nr_of_moves = move_list.len();

    // Run perft for each of the moves.
    for i in 0..nr_of_moves {
        // Get the move to be executed and counted.
        let m = move_list.get_move(i);

        // If the move is legal...
        if board.make(m, mg) {
            // Then count the number of leaf nodes it generates...
            leaf_nodes += perft(board, depth - 1, mg);

            // Then unmake the move so the next one can be counted.
            board.unmake();
        }
    }

    // Return the number of leaf nodes for the given position and depth.
    leaf_nodes
}
