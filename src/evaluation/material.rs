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

use super::defs::PIECE_VALUES;
use crate::{board::Board, defs::Sides, misc::bits};

pub fn count(board: &Board) -> (u16, u16) {
    let mut white_material: u16 = 0;
    let mut black_material: u16 = 0;
    let bb_w = board.bb_pieces[Sides::WHITE];
    let bb_b = board.bb_pieces[Sides::BLACK];

    for (piece, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
        let mut white_pieces = *w;
        let mut black_pieces = *b;

        while white_pieces > 0 {
            white_material += PIECE_VALUES[piece];
            bits::next(&mut white_pieces);
        }

        while black_pieces > 0 {
            black_material += PIECE_VALUES[piece];
            bits::next(&mut black_pieces);
        }
    }

    (white_material, black_material)
}
