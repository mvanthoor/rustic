// search.rs contains the engine's search routine.

pub mod sorting;

use crate::{
    board::{defs::SQUARE_NAME, Board},
    defs::MAX_DEPTH,
    engine::defs::{ErrFatal, Information},
    evaluation,
    movegen::{
        defs::{Move, MoveList, MoveType},
        MoveGenerator,
    },
};
use crossbeam_channel::{Receiver, Sender};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

const INF: i16 = 25000;
const CHECKMATE: i16 = 24000;
const STALEMATE: i16 = 0;

#[derive(PartialEq)]
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

pub struct SearchParams {
    depth: u8,
}

impl SearchParams {
    pub fn new(depth: u8) -> Self {
        Self { depth }
    }
}

#[derive(PartialEq)]
pub struct SearchInfo {
    pub best_move: Move,
    pub termination: SearchTerminate,
    pub nodes: usize,
    pub ply: u8,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            best_move: Move::new(0),
            termination: SearchTerminate::Nothing,
            nodes: 0,
            ply: 0,
        }
    }
}

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
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        mg: Arc<MoveGenerator>,
    ) {
        // Set up a channel for incoming commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Create thread-local variables.
        let _t_report_tx = report_tx.clone();
        let t_mg = Arc::clone(&mg);
        let t_arc_board = Arc::clone(&board);
        let mut t_search_info = SearchInfo::new();

        // Create the search thread.
        let h = thread::spawn(move || {
            let mut quit = false;
            let mut halt = true;

            while !quit {
                let cmd = control_rx.recv().expect(ErrFatal::CHANNEL);

                match cmd {
                    SearchControl::Start => {
                        halt = false;
                    }
                    SearchControl::Stop => halt = true,
                    SearchControl::Quit => quit = true,
                    SearchControl::Nothing => (),
                }

                if !halt && !quit {
                    let mtx_board = t_arc_board.lock().expect(ErrFatal::LOCK);
                    let mut board = mtx_board.clone();

                    let mut search_params = SearchParams::new(7);
                    t_search_info = SearchInfo::new();
                    t_search_info.termination = SearchTerminate::Nothing;

                    std::mem::drop(mtx_board);

                    Search::iterative_deepening(
                        &mut board,
                        &t_mg,
                        &mut search_params,
                        &mut t_search_info,
                        &control_rx,
                    );
                }

                match t_search_info.termination {
                    SearchTerminate::Stop => {
                        halt = true;
                    }
                    SearchTerminate::Quit => {
                        halt = true;
                        quit = true;
                    }
                    _ => (),
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
        board: &mut Board,
        mg: &Arc<MoveGenerator>,
        search_params: &mut SearchParams,
        search_info: &mut SearchInfo,
        control_rx: &Receiver<SearchControl>,
    ) {
        let mut depth = 1;
        let mut terminate = false;

        while depth <= search_params.depth && depth <= MAX_DEPTH && !terminate {
            let now = std::time::Instant::now();

            let eval = Search::alpha_beta(
                depth,
                -INF,
                INF,
                board,
                mg,
                search_params,
                search_info,
                control_rx,
            );

            let elapsed = now.elapsed().as_millis();
            let knps = ((search_info.nodes * 1000) as f64 / elapsed as f64).floor() as usize / 1000;

            println!(
                "depth: {}, best move: {}{}, eval: {}, nodes: {}, knps: {}",
                depth,
                SQUARE_NAME[search_info.best_move.from()],
                SQUARE_NAME[search_info.best_move.to()],
                eval,
                search_info.nodes,
                knps
            );
            depth += 1;
        }
    }

    fn alpha_beta(
        depth: u8,
        mut alpha: i16,
        beta: i16,
        board: &mut Board,
        mg: &Arc<MoveGenerator>,
        search_params: &mut SearchParams,
        search_info: &mut SearchInfo,
        control_rx: &Receiver<SearchControl>,
    ) -> i16 {
        // Check for stop or quit commands.
        // ======================================================================

        /*

        let cmd = control_rx.try_recv().unwrap_or(SearchControl::Nothing);
        match cmd {
            SearchControl::Stop => {
                search_info.termination = SearchTerminate::Stop;
                return 0;
            }
            SearchControl::Quit => {
                search_info.termination = SearchTerminate::Quit;
                return 0;
            }
            _ => (),
        };

        */
        // ======================================================================

        // We have arrived at the leaf node. Evaluate the position and
        // return the result.
        if depth == 0 {
            return evaluation::evaluate_position(board);
        }

        // Temporary variables.
        let mut current_best_move = Move::new(0);
        let old_alpha = alpha;

        // Search a new node, so we increase the node counter.
        search_info.nodes += 1;

        // Generate the moves in this position
        let mut legal_moves_found = 0;
        let mut move_list = MoveList::new();
        mg.generate_moves(board, &mut move_list, MoveType::All);

        // Iterate over the moves.
        for i in 0..move_list.len() {
            let current_move = move_list.get_move(i);
            let is_legal = board.make(current_move, mg);

            // If not legal, skip the move and the rest of the function.
            if !is_legal {
                continue;
            }

            // At this point, a legal move was found.
            legal_moves_found += 1;

            // Move is legal; increase the ply count.
            search_info.ply += 1;

            // We are not yet in a leaf node (the "bottom" of the tree, at
            // the requested depth), so start Alpha-Beta again, for the
            // opponent's side to go one ply deeper.
            let eval_score = -Search::alpha_beta(
                depth - 1,
                -beta,
                -alpha,
                board,
                mg,
                search_params,
                search_info,
                control_rx,
            );

            // Take back the move, and decrease ply accordingly.
            board.unmake();
            search_info.ply -= 1;

            // Beta-cut-off. We return this score, because searching any
            // further down this path would make the situation worse for us
            // and better for our opponent. This is called "fail-high".
            if eval_score >= beta {
                return beta;
            }

            // We found a better move for us.
            if eval_score > alpha {
                // Save our better evaluation score.
                alpha = eval_score;
                current_best_move = current_move;
            }
        }

        // If we exit the loop without legal moves being found, the
        // side to move is either in checkmate or stalemate.
        if legal_moves_found == 0 {
            let king_square = board.king_square(board.us());
            let is_in_check = mg.square_attacked(board, board.opponent(), king_square);

            if is_in_check {
                // It is minus checkmate (negative), because when checkmate
                // is detected for the player to move, this is a very bad
                // situation for that player.
                return -CHECKMATE + (search_info.ply as i16);
            } else {
                return STALEMATE;
            }
        }

        // Alpha was improved while walking through the move list, so a
        // better move was found.
        if alpha != old_alpha {
            search_info.best_move = current_best_move;
        }

        // We have traversed the entire move list and found the best
        // possible move/eval_score for us at this depth, so return it.
        // This called "fail low".
        return alpha;
    }
}
