use crate::search::Search;
use crate::{
    basetypes::error::ErrFatal,
    board::Board,
    communication::defs::EngineInput,
    movegen::MoveGenerator,
    search::defs::{
        SearchControl, SearchInfo, SearchParams, SearchRefs, SearchReport, SearchTerminated,
    },
};
use std::{
    sync::{Arc, Mutex, mpsc::Sender, mpsc::channel},
    thread::{self},
};

impl Search {
    pub fn init(
        &mut self,
        report_tx: Sender<EngineInput>, // Used to send information to engine.
        board: Arc<Mutex<Board>>,       // Arc pointer to engine's board.
        mg: Arc<MoveGenerator>,         // Arc pointer to engine's move generator.
    ) {
        // Set up a channel for incoming commands
        let (control_tx, control_rx) = channel();

        // Create thread-local variables.
        let t_report_tx = report_tx;
        let arc_board = Arc::clone(&board);
        let arc_mg = Arc::clone(&mg);
        let arc_tt = Arc::clone(&self.transposition);

        // Create the search thread.
        let h = thread::spawn(move || {
            let mut search_params = SearchParams::new();
            let mut quit = false;
            let mut halt = true;

            // As long as the search isn't quit, keep this thread alive.
            while !quit {
                // Inform the engine that we are now ready to search.
                t_report_tx
                    .send(EngineInput::Search(SearchReport::Ready))
                    .expect(ErrFatal::CHANNEL);

                // Wait for the next incoming command from the engine.
                let cmd = control_rx.recv().expect(ErrFatal::CHANNEL);

                // And react accordingly.
                match cmd {
                    SearchControl::Start(sp) => {
                        search_params = sp;
                        halt = false; // This will start the search.
                    }
                    SearchControl::Stop | SearchControl::Abandon => halt = true,
                    SearchControl::Quit => quit = true,
                    SearchControl::Nothing => (),
                }

                // Search isn't halted and not going to quit.
                if !halt && !quit {
                    // Copy the current board to be used in this thread.
                    let mtx_board = arc_board.lock().expect(ErrFatal::LOCK);
                    let mut board = mtx_board.clone();
                    std::mem::drop(mtx_board);

                    // Create a place to put search information
                    let mut search_info = SearchInfo::new();

                    // Create references to all needed information and structures.
                    let mut search_refs = SearchRefs {
                        board: &mut board,
                        mg: &arc_mg,
                        transposition: &arc_tt,
                        search_params: &mut search_params,
                        search_info: &mut search_info,
                        control_rx: &control_rx,
                        report_tx: &t_report_tx,
                    };

                    // Start the search using Iterative Deepening.
                    let (best_move, termination) = Search::iterative_deepening(&mut search_refs);

                    // if we didn't abandon the search, return a best move.
                    if termination != SearchTerminated::Abandoned {
                        let info = EngineInput::Search(SearchReport::Finished(best_move));
                        t_report_tx.send(info).expect(ErrFatal::CHANNEL);
                    }

                    // We halt (stop) or quit the search according to the
                    // termination condition.
                    match termination {
                        SearchTerminated::Stopped | SearchTerminated::Abandoned => {
                            halt = true;
                        }
                        SearchTerminated::Quit => {
                            halt = true;
                            quit = true;
                        }
                        SearchTerminated::Nothing => (),
                    }
                }
            }
        });

        // Store the thread's handle and command sender.
        self.handle = Some(h);
        self.control_tx = Some(control_tx);
    }

    pub fn transposition_clear(&self) {
        self.transposition.lock().expect(ErrFatal::LOCK).clear();
    }

    pub fn transposition_resize(&self, megabytes: usize) {
        self.transposition
            .lock()
            .expect(ErrFatal::LOCK)
            .resize(megabytes);
    }

    // This function is used to send commands into the search thread.
    pub fn send(&self, cmd: SearchControl) {
        if let Some(tx) = &self.control_tx {
            tx.send(cmd).expect(ErrFatal::CHANNEL);
        }
    }

    // After sending the quit command, the engine calls this function to
    // wait for the search to shut down.
    pub fn shutdown(&mut self) {
        if let Some(h) = self.handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }
}
