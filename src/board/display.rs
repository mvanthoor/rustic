use crate::{
    board::defs::{RangeOf, SQUARE_NAME},
    defs::{Bitboard, NrOf, Sides},
};

use super::Board;
use std::fmt::{self, Display};

type AsciiBoard = [char; NrOf::SQUARES];

const CHAR_ZERO: char = '0';
const CHAR_ONE: char = '1';
const CHAR_EMPTY_SQUARE: char = '.';
const PIECE_CHARS: [[char; NrOf::PIECE_TYPES]; Sides::BOTH] = [
    ['K', 'Q', 'R', 'B', 'N', 'I'],
    ['k', 'q', 'r', 'b', 'n', 'i'],
];

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ascii = self.bitboards_to_ascii();
        let pretty = ascii_board_to_pretty_string(&ascii);
        let meta = self.metadata_to_pretty_string();

        write!(f, "{}{}", pretty, meta)
    }
}

impl Board {
    pub fn bitboard_to_pretty_string(&self, b: Bitboard) -> String {
        let mut ascii_board: AsciiBoard = [CHAR_ZERO; NrOf::SQUARES];
        self.put_character_on_square(b, &mut ascii_board, CHAR_ONE);
        ascii_board_to_pretty_string(&ascii_board)
    }

    // Create a printable ASCII-board out of bitboards.
    fn bitboards_to_ascii(&self) -> AsciiBoard {
        let mut ascii_board: AsciiBoard = [CHAR_EMPTY_SQUARE; NrOf::SQUARES];

        for (piece, bb) in self.bb_pieces[Sides::WHITE].iter().enumerate() {
            self.put_character_on_square(*bb, &mut ascii_board, PIECE_CHARS[Sides::WHITE][piece]);
        }

        for (piece, bb) in self.bb_pieces[Sides::BLACK].iter().enumerate() {
            self.put_character_on_square(*bb, &mut ascii_board, PIECE_CHARS[Sides::BLACK][piece]);
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
    fn metadata_to_pretty_string(&self) -> String {
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
