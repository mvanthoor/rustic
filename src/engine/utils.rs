/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

use super::{
    defs::{ErrFatal, GameEndReason, GameResult},
    Engine,
};
use crate::{
    board::Board,
    comm::defs::CommOut,
    defs::{EngineRunResult, FEN_KIWIPETE_POSITION},
    misc::parse::PotentialMove,
    misc::{parse, result},
    movegen::{
        defs::{Move, MoveList, MoveType},
        MoveGenerator,
    },
};
use if_chain::if_chain;
use std::sync::Mutex;

impl Engine {
    // This function sets up a position using a given FEN-string.
    pub fn setup_position(&mut self) -> EngineRunResult {
        // Get either the provided FEN-string or KiwiPete. If both are
        // provided, the KiwiPete position takes precedence.
        let f = &self.cmdline.fen()[..];
        let kp = self.cmdline.has_kiwipete();
        let fen = if kp { FEN_KIWIPETE_POSITION } else { f };

        // Lock the board, setup the FEN-string, and drop the lock.
        self.board
            .lock()
            .expect(ErrFatal::LOCK)
            .fen_read(Some(fen))?;

        Ok(())
    }

    // This function executes a move on the internal board, if it legal to
    // do so in the given position.
    pub fn execute_move(&mut self, m: String) -> bool {
        // Prepare shorthand variables.
        let empty = (0usize, 0usize, 0usize);
        let potential_move = parse::algebraic_move_to_number(&m[..]).unwrap_or(empty);
        let is_pseudo_legal = self.pseudo_legal(potential_move, &self.board, &self.mg);
        let mut is_legal = false;

        if let Ok(ips) = is_pseudo_legal {
            is_legal = self.board.lock().expect(ErrFatal::LOCK).make(ips, &self.mg);
        }
        is_legal
    }

    // After the engine receives an incoming move, it checks if this move
    // is actually in the list of pseudo-legal moves for this position.
    pub fn pseudo_legal(
        &self,
        m: PotentialMove,
        board: &Mutex<Board>,
        mg: &MoveGenerator,
    ) -> Result<Move, ()> {
        let mut result = Err(());

        // Get the pseudo-legal move list for this position.
        let mut ml = MoveList::new();
        let mtx_board = board.lock().expect(ErrFatal::LOCK);
        mg.generate_moves(&mtx_board, &mut ml, MoveType::All);
        std::mem::drop(mtx_board);

        // Determine if the potential move is pseudo-legal. make() wil
        // determine final legality when executing the move.
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

    // This function sends the game result (and the reason for that result)
    //  to the output thread. If the current protocol requires it, the
    //  result will be sent to the GUI.
    pub fn send_game_result(&mut self) -> GameEndReason {
        // Lock the board and determine the game's end result and reason.
        let mut mtx_board = self.board.lock().expect(ErrFatal::LOCK);
        let game_end_reason = result::game_end_reason(&mut mtx_board, &self.mg);

        match game_end_reason {
            // The game is still going. We don't send anything.
            GameEndReason::NotEnded => (),

            // Side to move is checkmated.
            GameEndReason::Checkmate => {
                // If checkmated and we are white, then black wins.
                if mtx_board.us_is_white() {
                    self.comm
                        .send(CommOut::Result(GameResult::BlackWins, game_end_reason));
                } else {
                    // And the other way around, obviously.
                    self.comm
                        .send(CommOut::Result(GameResult::WhiteWins, game_end_reason));
                }
            }

            // A draw is a draw, irrespective of side to move.
            GameEndReason::Stalemate
            | GameEndReason::Insufficient
            | GameEndReason::FiftyMoves
            | GameEndReason::ThreeFold => {
                self.comm
                    .send(CommOut::Result(GameResult::Draw, game_end_reason));
            }
        }

        game_end_reason
    }
}
