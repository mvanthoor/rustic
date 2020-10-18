// search.rs contains the engine's search routine.

mod alpha_beta;
pub mod defs;
mod sorting;

use crate::{
    board::{defs::SQUARE_NAME, Board},
    defs::MAX_DEPTH,
    engine::defs::{ErrFatal, Information},
    movegen::MoveGenerator,
};
use crossbeam_channel::Sender;
use defs::{
    SearchControl, SearchInfo, SearchParams, SearchRefs, SearchReport, SearchTerminate, INF,
};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct Search {
    handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<SearchControl>>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            handle: None,
            control_tx: None,
        }
    }

    pub fn init(
        &mut self,
        report_tx: Sender<Information>, // Used to send information to engine.
        board: Arc<Mutex<Board>>,       // Arc pointer to engine's board.
        mg: Arc<MoveGenerator>,         // Arc pointer to engine's move generator.
    ) {
        // Set up a channel for incoming commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Create thread-local variables.
        let t_report_tx = report_tx.clone();

        // Create the search thread.
        let h = thread::spawn(move || {
            // Pointer to Board and Move Generator for this thread.
            let arc_board = Arc::clone(&board);
            let arc_mg = Arc::clone(&mg);

            let mut quit = false;
            let mut halt = true;

            // As long as the search isn't quit, keep this thread alive.
            while !quit {
                // Wait for the next incoming command from the engine.
                let cmd = control_rx.recv().expect(ErrFatal::CHANNEL);

                // And react accordingly.
                match cmd {
                    SearchControl::Start => {
                        halt = false; // This will start the search.
                    }
                    SearchControl::Stop => halt = true,
                    SearchControl::Quit => quit = true,
                    SearchControl::Nothing => (),
                }

                // Search isn't halted and not going to quit.
                if !halt && !quit {
                    // Copy the current board to be used in this thread.
                    let mtx_board = arc_board.lock().expect(ErrFatal::LOCK);
                    let mut board = mtx_board.clone();
                    std::mem::drop(mtx_board);

                    // Set up search parameters.
                    let mut search_params = SearchParams::new(MAX_DEPTH, 1000 * 60 * 2);
                    let mut search_info = SearchInfo::new();
                    search_info.terminate = SearchTerminate::Nothing;

                    // Create references to all needed information.
                    let mut search_refs = SearchRefs {
                        board: &mut board,                 // Just copied board.
                        mg: &arc_mg,                       // Move generator within engine.
                        search_params: &mut search_params, // Search parameters.
                        search_info: &mut search_info,     // A place to put search results.
                        control_rx: &control_rx,           // This thread's command receiver.
                    };

                    // Start the search using Iterative Deepening.
                    Search::iterative_deepening(&mut search_refs);

                    // Inform the engine that the search has finished.
                    let best_move = search_info.curr_move;
                    let information = Information::Search(SearchReport::Finished(best_move));
                    t_report_tx.send(information).expect(ErrFatal::CHANNEL);

                    // If the search was finished due to a Stop or Quit
                    // command then either halt or quit the search.
                    match search_info.terminate {
                        SearchTerminate::Stop => {
                            halt = true;
                        }
                        SearchTerminate::Quit => {
                            halt = true;
                            quit = true;
                        }
                        SearchTerminate::Nothing => (),
                    }
                }
            }
        });

        // Store the thread's handle and command sender.
        self.handle = Some(h);
        self.control_tx = Some(control_tx);
    }

    // This function is used to send commands into the search thread.
    pub fn send(&self, cmd: SearchControl) {
        if let Some(tx) = &self.control_tx {
            tx.send(cmd).expect(ErrFatal::CHANNEL);
        }
    }

    // After sending the quit command, the engine calls this function to
    // wait for the search to shut down.
    pub fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }
}

// Actual search routines.
impl Search {
    fn iterative_deepening(refs: &mut SearchRefs) {
        let mut depth = 1;
        let mut terminate = false;

        while depth <= refs.search_params.depth && depth < MAX_DEPTH && !terminate {
            let now = std::time::Instant::now();

            let eval = Search::alpha_beta(depth, -INF, INF, refs);

            // Terminate iterative deepning if requested.
            terminate = refs.search_info.terminate != SearchTerminate::Nothing;

            if !terminate {
                let mut knps = 0;
                let seconds = now.elapsed().as_millis() as f64 / 1000f64;
                if seconds > 0f64 {
                    let knodes = refs.search_info.nodes as f64 / 1000f64;
                    knps = (knodes / seconds).round() as usize;
                }

                println!(
                    "depth: {}, best move: {}{}, eval: {}, time: {}s, nodes: {}, knps: {}",
                    depth,
                    SQUARE_NAME[refs.search_info.curr_move.from()],
                    SQUARE_NAME[refs.search_info.curr_move.to()],
                    eval,
                    seconds,
                    refs.search_info.nodes,
                    knps
                );

                depth += 1;
            }
        }
    }
}
