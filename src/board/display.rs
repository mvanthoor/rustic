/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2022, Marcel Vanthoor
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
    board::defs::{RangeOf, SQUARE_NAME},
    defs::{Bitboard, NrOf, Sides},
};

use super::{defs::Pieces, Board};
use std::fmt::{self, Display};

type AsciiBoard = [char; NrOf::SQUARES];

const CHAR_ES: char = '.';
const CHAR_WK: char = 'K';
const CHAR_WQ: char = 'Q';
const CHAR_WR: char = 'R';
const CHAR_WB: char = 'B';
const CHAR_WN: char = 'N';
const CHAR_WP: char = 'I';
const CHAR_BK: char = 'k';
const CHAR_BQ: char = 'q';
const CHAR_BR: char = 'r';
const CHAR_BB: char = 'b';
const CHAR_BN: char = 'n';
const CHAR_BP: char = 'i';

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ascii = self.bitboards_to_ascii();
        let pretty = ascii_board_to_pretty_string(&ascii);
        let meta = self.metadata_to_string();

        write!(f, "{}{}", pretty, meta)
    }
}

impl Board {
    // Create a printable ASCII-board out of bitboards.
    fn bitboards_to_ascii(&self) -> AsciiBoard {
        let mut ascii_board: AsciiBoard = [CHAR_ES; NrOf::SQUARES];
        let bb_w = self.bb_pieces[Sides::WHITE];
        let bb_b = self.bb_pieces[Sides::BLACK];

        for (piece, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            match piece {
                Pieces::KING => {
                    self.put_character_on_square(*w, &mut ascii_board, CHAR_WK);
                    self.put_character_on_square(*b, &mut ascii_board, CHAR_BK);
                }
                Pieces::QUEEN => {
                    self.put_character_on_square(*w, &mut ascii_board, CHAR_WQ);
                    self.put_character_on_square(*b, &mut ascii_board, CHAR_BQ);
                }
                Pieces::ROOK => {
                    self.put_character_on_square(*w, &mut ascii_board, CHAR_WR);
                    self.put_character_on_square(*b, &mut ascii_board, CHAR_BR);
                }
                Pieces::BISHOP => {
                    self.put_character_on_square(*w, &mut ascii_board, CHAR_WB);
                    self.put_character_on_square(*b, &mut ascii_board, CHAR_BB);
                }
                Pieces::KNIGHT => {
                    self.put_character_on_square(*w, &mut ascii_board, CHAR_WN);
                    self.put_character_on_square(*b, &mut ascii_board, CHAR_BN);
                }
                Pieces::PAWN => {
                    self.put_character_on_square(*w, &mut ascii_board, CHAR_WP);
                    self.put_character_on_square(*b, &mut ascii_board, CHAR_BP);
                }
                _ => (),
            }
        }

        ascii_board
    }

    // This function actually puts the correct character into the ASCII board.
    fn put_character_on_square(
        &self,
        bitboard: Bitboard,
        ascii_board: &mut AsciiBoard,
        character: char,
    ) {
        for (i, square) in ascii_board.iter_mut().enumerate() {
            if (bitboard >> i) & 1 == 1 {
                *square = character;
            }
        }
    }

    // This function prints all of the metadata about the position.
    fn metadata_to_string(&self) -> String {
        let mut meta = String::from("");
        let zk = self.game_state.zobrist_key;
        let active_color = if self.is_white_to_move() {
            "White"
        } else {
            "Black"
        };
        let castling = self.game_state.castling_to_string();
        let en_passant = match self.game_state.en_passant {
            Some(ep) => SQUARE_NAME[ep as usize],
            None => "-",
        };
        let hmc = self.game_state.halfmove_clock;
        let fmn = self.game_state.fullmove_number;

        meta.push_str(format!("{:<20}{:x}\n", "Zobrist key:", zk).as_str());
        meta.push_str(format!("{:<20}{}\n", "Active Color:", active_color).as_str());
        meta.push_str(format!("{:<20}{}\n", "Castling:", castling).as_str());
        meta.push_str(format!("{:<20}{}\n", "En Passant:", en_passant).as_str());
        meta.push_str(format!("{:<20}{}\n", "Half-move clock:", hmc).as_str());
        meta.push_str(format!("{:<20}{}\n", "Full-move number:", fmn).as_str());

        meta
    }
}

// Convert an AsciiBoard to a string.
fn ascii_board_to_pretty_string(ascii_board: &AsciiBoard) -> String {
    let mut pretty = String::from("");
    let coordinate_alpha: &str = "ABCDEFGH";
    let mut coordinate_digit = NrOf::FILES;

    pretty += "\n";

    for current_rank in RangeOf::RANKS.rev() {
        pretty.push_str(format!("{}   ", coordinate_digit).as_str());
        for current_file in RangeOf::FILES {
            let square = (current_rank as usize * NrOf::FILES) + current_file as usize;
            let character = ascii_board[square];
            pretty.push_str(format!("{} ", character).as_str());
        }
        pretty += "\n";
        coordinate_digit -= 1;
    }

    pretty += "\n";
    pretty += str::repeat(" ", 4).as_str();

    for c in coordinate_alpha.chars() {
        pretty.push_str(format!("{} ", c).as_str());
    }

    pretty += "\n\n";

    pretty
}
