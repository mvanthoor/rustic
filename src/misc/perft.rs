use crate::{
    board::Board,
    misc::print,
    movegen::{
        defs::{Move, MoveList},
        MoveGenerator,
    },
};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Instant,
};

struct PerftChunk {
    ml: MoveList,
    handle: Option<JoinHandle<u64>>,
}

impl PerftChunk {
    pub fn new(ml: MoveList) -> Self {
        Self { ml, handle: None }
    }

    pub fn add_move(&mut self, m: Move) {
        self.ml.push(m);
    }

    pub fn get_move_list(&self) -> MoveList {
        self.ml
    }

    pub fn set_handle(&mut self, h: JoinHandle<u64>) {
        self.handle = Some(h);
    }

    pub fn get_result(&mut self) -> u64 {
        let mut result = 0;
        if let Some(h) = self.handle.take() {
            result = h.join().unwrap_or(0)
        }
        result
    }
}

// This function runs perft(), while collecting speed information.
// It uses iterative deepening, so when running perft(7), it will output
// the results of perft(1) up to and including perft(7).
pub fn run(board: Arc<Mutex<Board>>, depth: u8, threads: u8, mg: Arc<MoveGenerator>) {
    let mut total_time: u128 = 0;
    let mut total_nodes: u64 = 0;

    // Create a mutex guard for the board, so it can be safely cloned.
    // Panic if the guard can't be created, because something is wrong with
    // the main engine thread.
    let guard_board = board.lock().expect("Perft: board lock failed.");

    // Clone the board using the guard.
    let local_board = guard_board.clone();

    // The function now has its own local board. Drop the guard. It is not
    // necessary to keep the lock until perft runs out.
    std::mem::drop(guard_board);

    println!("Benchmarking perft 1-{} on {} threads", depth, threads);

    print::position(&local_board, None);

    // Perform all perfts for depths 1 up to and including "depth"
    for d in 1..=depth {
        // Create perft chunks: 1 for each thread. Each chunk contains a
        // part of the initial movelist of the position, and a handle for
        // the thread that will run on it.
        let mut chunks = create_chunks(&local_board, threads, &mg.clone());

        // Current time
        let now = Instant::now();
        let mut leaf_nodes = 0;

        // Iterate over the chunks, and launch a thread for each.
        for chunk in chunks.iter_mut() {
            // Each thread needs its own data to work with, so we create
            // some variables that are local to this for-loop.
            let mut thread_board: Board = local_board.clone(); // clones a real board.
            let thread_mg = mg.clone(); // clones the Arc<> pointer only.
            let thread_ml = chunk.get_move_list(); // gets a real move list.

            // Create and launch a thread for the current chunk. The thread
            // will take control of the local for-loop variables. (The next
            // loop iteration that launches a thread will have new
            // variables, coming from the next chunk.)
            chunk.set_handle(thread::spawn(move || {
                // Run perft on the move list in this chunk.
                perft_on_chunk(&mut thread_board, d, &thread_ml, &thread_mg)
            }));
        }

        // Iterate over the chunks again, and get the results of each.
        for chunk in chunks.iter_mut() {
            leaf_nodes += chunk.get_result();
        }

        // Measure time and speed
        let elapsed = now.elapsed().as_millis();
        let leaves_per_second = ((leaf_nodes * 1000) as f64 / elapsed as f64).floor();

        // Add tot totals for final calculation at the very end.
        total_time += elapsed;
        total_nodes += leaf_nodes;

        // Print the results.
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

// This is the actual Perft function. It is public, because it is used by
// the "testsuite" module.
pub fn perft(board: &mut Board, depth: u8, mg: &MoveGenerator) -> u64 {
    let mut leaf_nodes: u64 = 0;
    let mut move_list: MoveList = MoveList::new();

    // Count each visited leaf node.
    if depth == 0 {
        return 1;
    }

    mg.gen_all_moves(board, &mut move_list);

    // Run perft for each of the moves.
    for i in 0..move_list.len() {
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

// This function creates the initial move list and one chunk for each
// thread. Then it will distribute the moves across these chunks.
fn create_chunks(board: &Board, threads: u8, mg: &MoveGenerator) -> Vec<PerftChunk> {
    // This vector holds one chunk per thread.
    let mut chunk_list: Vec<PerftChunk> = Vec::with_capacity(threads as usize);

    // Actually create one empty chunk per thread.
    for _ in 0..threads {
        chunk_list.push(PerftChunk::new(MoveList::new()));
    }

    // Create an movelist to divide into chunks.
    let mut move_list = MoveList::new();
    mg.gen_all_moves(board, &mut move_list);

    // Loop through the initial move list, and distribute the moves over
    // the chunks. (20 moves, with 4 threads, will yield 4 chunks with 5
    // moves each. In case of 21 moves, the first chunk will get the one
    // extra move, and so on.)
    let mut current_chunk: usize = 0;
    for i in 0..move_list.len() {
        let m = move_list.get_move(i);
        chunk_list[current_chunk].add_move(m);
        current_chunk += 1;
        if current_chunk == threads as usize {
            current_chunk = 0;
        }
    }

    chunk_list
}

// This function is a "mini-perft". It receives a move list, which is a
// part of the full move list of the position at the first ply. (This move
// list comes from one of the chunks created in the function above.) Perft
// will run on this partial move list. Receiving a move list only happens
// on the ver first ply: by using this function to handle the first ply,
// perft() itself can omit the check where it determines if it received a
// move list, or if it needs to create one from scratch.
fn perft_on_chunk(board: &mut Board, depth: u8, ml: &MoveList, mg: &MoveGenerator) -> u64 {
    let mut leaf_nodes = 0;

    // Iterate over each of the moves in the list.
    for i in 0..ml.len() {
        let m = ml.get_move(i);

        // If it is a legal move...
        if board.make(m, mg) {
            // ...then run perft on it to calculate the number of leaf nodes...
            leaf_nodes += perft(board, depth - 1, mg);

            // Then unmake the move and do the next one.
            board.unmake();
        }
    }

    leaf_nodes
}
