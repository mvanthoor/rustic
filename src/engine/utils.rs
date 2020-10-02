use super::{Engine, ErrFatal};
use crate::{
    board::Board,
    defs::{About, EngineRunResult, FEN_KIWIPETE_POSITION},
    misc::parse::PotentialMove,
    movegen::{
        defs::{Move, MoveList},
        MoveGenerator,
    },
};
use if_chain::if_chain;
use std::sync::Mutex;

// This notice is displayed if the engine is a debug binary. (Debug
// binaries are unoptimized and slower than release binaries.)
#[cfg(debug_assertions)]
const NOTICE_DEBUG_MODE: &str = "Notice: Running in debug mode";

impl Engine {
    pub fn ascii_logo(&self) {
        println!();
        println!("d888888b                      dP   oo        ");
        println!("88     88                     88             ");
        println!("88oooo88  88    88  d8888b  d8888P dP d88888b");
        println!("88    88  88    88  8ooooo    88   88 88     ");
        println!("88     88 88    88       88   88   88 88     ");
        println!("88     88  88888P  888888P    dP   dP 888888P");
        println!("ooooooooooooooooooooooooooooooooooooooooooooo");
        println!();
    }

    // Print information about the engine.
    pub fn about(&self) {
        println!("Program: {} {}", About::ENGINE, About::VERSION);
        println!("Author: {} <{}>", About::AUTHOR, About::EMAIL);
        println!("Website: {}", About::WEBSITE);
        println!(
            "Threads: {} (not used yet, always 1)",
            self.settings.threads
        );
        println!("Protocol: {}", self.comm.get_protocol_name());

        #[cfg(debug_assertions)]
        println!("{}", NOTICE_DEBUG_MODE);
    }

    // This function sets up a position using a given FEN-string.
    pub fn setup_position(&mut self) -> EngineRunResult {
        // Get either the provided FEN-string or KiwiPete. If both are
        // provided, the KiwiPete position takes precedence.
        let f = &self.cmdline.fen()[..];
        let kp = self.cmdline.has_kiwipete();
        let fen = if kp { FEN_KIWIPETE_POSITION } else { f };

        // Lock the board, setup the FEN-string, and drop the lock.
        let mut mtx_board = self.board.lock().expect(ErrFatal::LOCK);
        mtx_board.fen_read(Some(fen))?; // Abort if setup fails.
        std::mem::drop(mtx_board);

        Ok(())
    }

    // After the engine receives an incoming move, it checks if this move
    // is actually possible in the current board position.
    pub fn pseudo_legal(
        &self,
        m: PotentialMove,
        board: &Mutex<Board>,
        mg: &MoveGenerator,
    ) -> Result<Move, ()> {
        let mut result = Err(());
        let mut ml = MoveList::new();
        let mtx_board = board.lock().expect(ErrFatal::LOCK);

        mg.gen_all_moves(&mtx_board, &mut ml);
        std::mem::drop(mtx_board);

        // See if the provided potential move is a pseudo-legal move.
        // make() will later determine final legality, i.e. if the king is
        // left in check.
        for i in 0..ml.len() {
            let current = ml.get_move(i);
            if_chain! {
                if m.0 == current.from();
                if m.1 == current.to();
                if m.2 == current.promoted();
                then {
                    result = Ok(current);
                    break;
                }
            }
        }

        result
    }
}
