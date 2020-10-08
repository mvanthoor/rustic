// search.rs contains the engine's search routine.

use crate::{
    board::Board,
    defs::MAX_DEPTH,
    engine::defs::{ErrFatal, Information},
};
use crossbeam_channel::{Receiver, Sender};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub enum SearchControl {
    Start,
    Stop,
    Quit,
    Nothing,
}

#[derive(PartialEq)]
pub enum SearchTerminate {
    Stop,
    Quit,
    Nothing,
}

pub struct SearchInfo {
    pub termination: SearchTerminate,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            termination: SearchTerminate::Nothing,
        }
    }
}

pub struct Search {
    handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<SearchControl>>,
    search_info: Arc<Mutex<SearchInfo>>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            handle: None,
            control_tx: None,
            search_info: Arc::new(Mutex::new(SearchInfo::new())),
        }
    }

    pub fn init(&mut self, report_tx: Sender<Information>, board: Arc<Mutex<Board>>) {
        // Set up a channel for incoming commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Create thread-local variables.
        let _t_report_tx = report_tx.clone();
        let _t_arc_board = Arc::clone(&board);
        let t_search_info = Arc::clone(&self.search_info);

        // Create the search thread.
        let h = thread::spawn(move || {
            let mut quit = false;
            let mut halt = true;

            while !quit {
                let cmd = control_rx.recv().expect(ErrFatal::CHANNEL);

                match cmd {
                    SearchControl::Start => {
                        let mut mtx_si = t_search_info.lock().expect(ErrFatal::LOCK);
                        mtx_si.termination = SearchTerminate::Nothing;
                        halt = false;
                    }
                    SearchControl::Stop => halt = true,
                    SearchControl::Quit => quit = true,
                    SearchControl::Nothing => (),
                }

                if !halt && !quit {
                    Search::iterative_deepening(&t_search_info, &control_rx);
                }

                let mtx_si = t_search_info.lock().expect(ErrFatal::LOCK);
                match mtx_si.termination {
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

    pub fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }
}

// Actual search routines.
impl Search {
    fn iterative_deepening(
        search_info: &Arc<Mutex<SearchInfo>>,
        control_rx: &Receiver<SearchControl>,
    ) {
        let mut depth = 1;

        while depth <= MAX_DEPTH {
            Search::alpha_beta(depth, search_info, control_rx);

            let mtx_si = search_info.lock().expect(ErrFatal::LOCK);
            if mtx_si.termination != SearchTerminate::Nothing {
                return;
            }
            std::mem::drop(mtx_si);

            depth += 1;
        }
    }

    fn alpha_beta(
        depth: u8,
        search_info: &Arc<Mutex<SearchInfo>>,
        control_rx: &Receiver<SearchControl>,
    ) {
        if depth == 0 {
            println!("done.");
            return;
        }

        let cmd = control_rx.try_recv().unwrap_or(SearchControl::Nothing);
        let mut mtx_si = search_info.lock().expect(ErrFatal::LOCK);

        match cmd {
            SearchControl::Quit => {
                mtx_si.termination = SearchTerminate::Quit;
                return;
            }
            SearchControl::Stop => {
                mtx_si.termination = SearchTerminate::Stop;
                return;
            }
            _ => (),
        }

        std::mem::drop(mtx_si);

        println!("Depth: {}", depth);
        thread::sleep(std::time::Duration::from_secs(2));
        Search::alpha_beta(depth - 1, search_info, control_rx);
    }
}
