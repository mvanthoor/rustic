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
const NOTICE_DEBUG_MODE: &'static str = "Notice: Running in debug mode";

impl Engine {
    // Print information about the engine.
    pub fn about(&self) {
        println!("Program: {} {}", About::ENGINE, About::VERSION);
        println!("Author: {} <{}>", About::AUTHOR, About::EMAIL);
        println!("Description: {}", About::DESCRIPTION);
        println!(
            "Threads: {} (not used yet, always 1)",
            self.settings.threads
        );
        println!("Protocol: {}", self.comm.get_protocol_name());

        #[cfg(debug_assertions)]
        println!("{}", NOTICE_DEBUG_MODE);
    }

    pub fn setup_position(&mut self) -> EngineRunResult {
        // Get either the provided FEN-string or KiwiPete. If both are
        // provided, the KiwiPete position takes precedence.
        let f = &self.cmdline.fen()[..];
        let kp = self.cmdline.has_kiwipete();
        let fen = if kp { FEN_KIWIPETE_POSITION } else { f };

        // Lock the board, setup the FEN-string, and drop the lock.
        let mut mtx_board = self.board.lock().expect(ErrFatal::BOARD_LOCK);
        mtx_board.fen_read(Some(fen))?;
        std::mem::drop(mtx_board);

        Ok(())
    }

    pub fn pseudo_legal(
        &self,
        m: PotentialMove,
        board: &Mutex<Board>,
        mg: &MoveGenerator,
    ) -> Result<Move, ()> {
        let mut result = Err(());
        let mut ml = MoveList::new();
        let mtx_board = board.lock().expect(ErrFatal::BOARD_LOCK);

        mg.gen_all_moves(&mtx_board, &mut ml);
        std::mem::drop(mtx_board);

        // See if the provided potential move is a pseudo-legal move.
        // make() will later determine final legality (king in check).
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
