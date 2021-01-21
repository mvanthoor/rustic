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

use crate::board::defs::{Pieces, SQUARE_NAME};
use crate::defs::{Piece, Square};
use if_chain::if_chain;

pub type PotentialMove = (Square, Square, Piece);
pub type ParseMoveResult = Result<PotentialMove, ()>;

pub fn algebraic_move_to_number(m: &str) -> ParseMoveResult {
    let lower_case_move = m.to_ascii_lowercase();
    let mut potential_move: PotentialMove = (0, 0, Pieces::NONE);

    // Assume parsing the move will fail.
    let mut parse_move_result: ParseMoveResult = Err(());

    // Get the "from" and "to" squares from the move stirng.
    if m.len() == 4 || m.len() == 5 {
        if_chain! {
            // If converstion from algebraic square to number succeeds...
            if let Some(f) = algebraic_square_to_number(&lower_case_move[0..=1]);
            if let Some(t) = algebraic_square_to_number(&lower_case_move[2..=3]);
            then {
                // ...save the result
                potential_move.0 = f;
                potential_move.1 = t;

                // Up to here, parsing is OK.
                parse_move_result = Ok(potential_move);
            }
        };
    }

    // If Ok and there are 5 characters, keep parsing...
    if parse_move_result != Err(()) && m.len() == 5 {
        // Again, assume that parsing will fail.
        parse_move_result = Err(());

        // Get the promotion piece character.
        let c = lower_case_move.chars().last().unwrap_or('-');

        // If the conversion from character to promotion piece succeeds...
        if let Some(p) = promotion_piece_letter_to_number(c) {
            // ...save the result
            potential_move.2 = p;

            // and set the parsing result to Ok again.
            parse_move_result = Ok(potential_move);
        }
    }

    parse_move_result
}

pub fn algebraic_square_to_number(algebraic_square: &str) -> Option<Square> {
    // Convert String to &str.
    let a = &algebraic_square[..];
    // Get the index, which is also the square number.
    // If the square is not found, None is returned.
    SQUARE_NAME.iter().position(|&element| element == a)
}

#[allow(dead_code)]
pub fn promotion_piece_letter_to_number(piece_letter: char) -> Option<Piece> {
    // Assume that the character does not represent a promotion piece.
    // Note that this is NOT 'no piece' as in Pieces::NONE! This is 'no
    // piece' als in "no piece found for the provided character". This
    // happens if the character is 'a' or 'z' for example.
    let mut piece: Option<Piece> = None;

    // Convert the character to lowercase and get it for processing.
    if let Some(p) = piece_letter.to_lowercase().next() {
        // Set the promotion piece if the character is correct.
        match p {
            'q' => piece = Some(Pieces::QUEEN),
            'r' => piece = Some(Pieces::ROOK),
            'b' => piece = Some(Pieces::BISHOP),
            'n' => piece = Some(Pieces::KNIGHT),
            _ => (),
        }
    }
    // Return the piece if found, or None.
    piece
}
