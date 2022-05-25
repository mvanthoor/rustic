use crate::movegen::defs::{MoveList, MoveType};

use super::{
    defs::{ErrFatal, GameResult, GameResultPoints, GameResultReason},
    Engine,
};

impl Engine {
    pub fn game_over(&self) -> Option<GameResult> {
        let mut points = GameResultPoints::Nothing;
        let mut reason = GameResultReason::Nothing;
        let moves_available = self.moves_available();
        let mtx_board = self.board.lock().expect(ErrFatal::LOCK);

        if !moves_available {
            // Without moves available, it's either checkmate or stalemate.
            if self.we_are_in_check() {
                if mtx_board.is_white_to_move() {
                    points = GameResultPoints::BlackWins;
                    reason = GameResultReason::BlackMates;
                } else {
                    points = GameResultPoints::WhiteWins;
                    reason = GameResultReason::WhiteMates;
                }
            } else {
                points = GameResultPoints::Draw;
                reason = GameResultReason::Stalemate;
            }
        } else {
            // Even with moves available, we could still have a draw.
            match () {
                _ if mtx_board.draw_by_insufficient_material_rule() => {
                    points = GameResultPoints::Draw;
                    reason = GameResultReason::Insufficient;
                }
                _ if mtx_board.draw_by_fifty_move_rule() => {
                    points = GameResultPoints::Draw;
                    reason = GameResultReason::FiftyMoves;
                }
                _ if mtx_board.draw_by_repetition_rule() >= 2 => {
                    points = GameResultPoints::Draw;
                    reason = GameResultReason::ThreeFold;
                }
                _ => (),
            }
        };

        // Return the result if the game is ended.
        if (points != GameResultPoints::Nothing) && (reason != GameResultReason::Nothing) {
            Some(GameResult { points, reason })
        } else {
            None
        }
    }

    // Determines if the side to move has at least one legal move.
    pub fn moves_available(&self) -> bool {
        let mut move_list = MoveList::new();
        let mut mtx_board = self.board.lock().expect(ErrFatal::LOCK);

        // Generate pseudo-legal moves.
        self.mg
            .generate_moves(&mtx_board, &mut move_list, MoveType::All);

        // We can break as soon as we find a legal move.
        for i in 0..move_list.len() {
            let m = move_list.get_move(i);
            if mtx_board.make(m, &self.mg) {
                // Unmake the move we just made.
                mtx_board.unmake();
                // Return true, as we have at least one move.
                return true;
            }
        }

        // No legal moves available.
        false
    }

    pub fn we_are_in_check(&self) -> bool {
        let mtx_board = self.board.lock().expect(ErrFatal::LOCK);
        self.mg.square_attacked(
            &mtx_board,
            mtx_board.opponent(),
            mtx_board.king_square(mtx_board.us()),
        )
    }
}
