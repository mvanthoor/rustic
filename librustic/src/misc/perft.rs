use crate::{
    board::defs::FenResult,
    board::Board,
    movegen::{
        defs::{MoveList, MoveType},
        MoveGenerator,
    },
    search::defs::{HashData, TT},
};
use if_chain::if_chain;
use std::{sync::Arc, time::Instant};

// Implement required data for Perft
#[derive(Copy, Clone, Default)]
pub struct PerftData {
    depth: i8,
    leaf_nodes: u64,
}

impl HashData for PerftData {
    fn empty() -> Self {
        Self::default()
    }

    fn depth(&self) -> i8 {
        self.depth
    }
}

impl PerftData {
    pub fn new(depth: i8, leaf_nodes: u64) -> Self {
        Self { depth, leaf_nodes }
    }

    pub fn get_leaf_nodes(&self, depth: i8) -> Option<u64> {
        if self.depth == depth {
            Some(self.leaf_nodes)
        } else {
            None
        }
    }
}

// This function runs perft(), while collecting speed information.
// It uses iterative deepening, so when running perft(7), it will output
// the results of perft(1) up to and including perft(7).
pub fn run(fen: &str, depth: i8, mg: Arc<MoveGenerator>, tt_size: usize) -> FenResult {
    // Setup everything.
    let mut total_time: u128 = 0;
    let mut total_nodes: u64 = 0;
    let mut hash_full = String::from("");
    let mut local_board = Board::new();
    let mut transposition = TT::<PerftData>::new(tt_size);
    let tt_enabled = tt_size > 0;
    local_board.fen_setup(Some(fen))?;

    println!("Benchmarking perft 1-{depth}:");
    println!("{local_board}");

    // Perform all perft for depths 1 up to and including "depth"
    for d in 1..=depth {
        // Current time
        let now = Instant::now();
        let mut leaf_nodes = 0;

        leaf_nodes += perft(&mut local_board, d, &mg, &mut transposition, tt_enabled);

        // Measure time and speed
        let elapsed = now.elapsed().as_millis();
        let leaves_per_second = ((leaf_nodes * 1000) as f64 / elapsed as f64).floor();

        // Add tot totals for final calculation at the very end.
        total_time += elapsed;
        total_nodes += leaf_nodes;

        // Request TT usage. (This is provided per mille as per UCI
        // spec, so divide by 10 to get the usage in percents.)
        if tt_enabled {
            hash_full = format!(", hash full: {}%", transposition.hash_full_percent());
        }

        // Print the results.
        println!(
            "Perft {d}: {leaf_nodes} ({elapsed} ms, {leaves_per_second} leaves/sec{hash_full})"
        );
    }

    // Final calculation of the entire time taken, and average speed of leaves/second.
    let final_lnps = ((total_nodes * 1000) as f64 / total_time as f64).floor();
    println!("Total time spent: {total_time} ms");
    println!("Execution speed: {final_lnps} leaves/second");

    Ok(())
}

// This is the actual Perft function. It is public, because it is used by
// the "testsuite" module.
pub fn perft(
    board: &mut Board,
    depth: i8,
    mg: &MoveGenerator,
    transposition: &mut TT<PerftData>,
    tt_enabled: bool,
) -> u64 {
    let mut leaf_nodes: u64 = 0;

    // Count each visited leaf node.
    if depth == 0 {
        return 1;
    }

    // See if the current position is in the TT, and if so, get the
    // number of leaf nodes that were previously calculated for it.
    if_chain! {
        if tt_enabled;
        if let Some(data) = transposition.probe(board.game_state.zobrist_key);
        if let Some(leaf_nodes) = data.get_leaf_nodes(depth);
        then {
            return leaf_nodes;
        }
    }

    let mut move_list = MoveList::new();
    mg.generate_moves(board, &mut move_list, MoveType::All);

    // Run perft for each of the moves.
    for i in 0..move_list.len() {
        // Get the move to be executed and counted.
        let m = move_list.get_move(i);

        // If the move is legal...
        if board.make(m, mg) {
            // Then count the number of leaf nodes it generates...
            leaf_nodes += perft(board, depth - 1, mg, transposition, tt_enabled);

            // Then unmake the move so the next one can be counted.
            board.unmake();
        }
    }

    // We have calculated the number of leaf nodes for this position.
    // Store this in the TT for later use.
    if tt_enabled {
        transposition.insert(
            board.game_state.zobrist_key,
            PerftData::new(depth, leaf_nodes),
        )
    }

    // Return the number of leaf nodes for the given position and depth.
    leaf_nodes
}
