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

use crate::{
    board::Board,
    movegen::{
        defs::{MoveList, MoveType},
        MoveGenerator,
    },
};

// Determines if the side to move has at least one legal move.
pub fn has_moves(board: &mut Board, mg: &MoveGenerator) -> bool {
    let mut move_list = MoveList::new();

    // Generate pseudo-logal moves.
    mg.generate_moves(board, &mut move_list, MoveType::All);

    // We can break as soon as we find a legal move.
    for i in 0..move_list.len() {
        let m = move_list.get_move(i);
        if board.make(m, mg) {
            // Unmake the move we just made.
            board.unmake();
            // Return true, as we have at least one move.
            return true;
        }
    }

    // No legal moves available.
    false
}

pub fn in_check(board: &mut Board, mg: &MoveGenerator) -> bool {
    mg.square_attacked(board, board.opponent(), board.king_square(board.us()))
}
