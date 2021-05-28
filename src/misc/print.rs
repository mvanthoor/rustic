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
    board::{
        defs::{Pieces, RangeOf, PIECE_CHAR_CAPS, PIECE_NAME, SQUARE_NAME},
        Board,
    },
    defs::{Bitboard, Castling, NrOf, Sides},
    movegen::defs::{Move, MoveList},
};

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

// Prints the current position to the screen.
pub fn position(board: &Board, mark_square: Option<u8>) {
    let mut ascii_board: AsciiBoard = [CHAR_ES; NrOf::SQUARES];

    bitboards_to_ascii(board, &mut ascii_board);
    to_console(&ascii_board, mark_square);
    metadata(board);
}

// Create a printable ASCII-board out of bitboards.
fn bitboards_to_ascii(board: &Board, ascii_board: &mut AsciiBoard) {
    let bb_w = board.bb_pieces[Sides::WHITE];
    let bb_b = board.bb_pieces[Sides::BLACK];

    for (piece, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
        match piece {
            Pieces::KING => {
                put_character_on_square(*w, ascii_board, CHAR_WK);
                put_character_on_square(*b, ascii_board, CHAR_BK);
            }
            Pieces::QUEEN => {
                put_character_on_square(*w, ascii_board, CHAR_WQ);
                put_character_on_square(*b, ascii_board, CHAR_BQ);
            }
            Pieces::ROOK => {
                put_character_on_square(*w, ascii_board, CHAR_WR);
                put_character_on_square(*b, ascii_board, CHAR_BR);
            }
            Pieces::BISHOP => {
                put_character_on_square(*w, ascii_board, CHAR_WB);
                put_character_on_square(*b, ascii_board, CHAR_BB);
            }
            Pieces::KNIGHT => {
                put_character_on_square(*w, ascii_board, CHAR_WN);
                put_character_on_square(*b, ascii_board, CHAR_BN);
            }
            Pieces::PAWN => {
                put_character_on_square(*w, ascii_board, CHAR_WP);
                put_character_on_square(*b, ascii_board, CHAR_BP);
            }
            _ => (),
        }
    }
}

// This function actually puts the correct character into the ASCII board.
fn put_character_on_square(bitboard: Bitboard, ascii_board: &mut AsciiBoard, character: char) {
    for (i, square) in ascii_board.iter_mut().enumerate() {
        if (bitboard >> i) & 1 == 1 {
            *square = character;
        }
    }
}

// Print the generated ASCII-board to the console. Optionally mark one square.
fn to_console(ascii_board: &AsciiBoard, mark_square: Option<u8>) {
    let coordinate_alpha: &str = "ABCDEFGH";
    let mut coordinate_digit = NrOf::FILES;

    println!();
    for current_rank in RangeOf::RANKS.rev() {
        print!("{}   ", coordinate_digit);
        for current_file in RangeOf::FILES {
            let square = (current_rank as usize * NrOf::FILES) + current_file as usize;
            let character = ascii_board[square];
            if let Some(m) = mark_square {
                if m == (square as u8) {
                    // \x1b[0;35m is magenta
                    print!("\x1b[0;35m{} \x1b[0m", character);
                } else {
                    print!("{} ", character);
                }
            } else {
                print!("{} ", character);
            }
        }
        println!();
        coordinate_digit -= 1;
    }
    println!();
    print!("    ");
    for c in coordinate_alpha.chars() {
        print!("{} ", c);
    }
    println!();
    println!();
}

// This function prints all of the metadata about the position.
fn metadata(board: &Board) {
    let is_white = (board.game_state.active_color as usize) == Sides::WHITE;
    let active_color = if is_white { "White" } else { "Black" };
    let castling = castling_as_string(board.game_state.castling);
    let en_passant = match board.game_state.en_passant {
        Some(ep) => SQUARE_NAME[ep as usize],
        None => "-",
    };
    let hmc = board.game_state.halfmove_clock;
    let fmn = board.game_state.fullmove_number;

    println!("{:<20}{:x}", "Zobrist key:", board.game_state.zobrist_key);
    println!("{:<20}{}", "Active Color:", active_color);
    println!("{:<20}{}", "Castling:", castling);
    println!("{:<20}{}", "En Passant:", en_passant);
    println!("{:<20}{}", "Half-move clock:", hmc);
    println!("{:<20}{}", "Full-move number:", fmn);
    println!();
}

// Converts castling permissions to a string.
pub fn castling_as_string(permissions: u8) -> String {
    let mut castling_as_string: String = String::from("");
    let p = permissions;

    castling_as_string += if p & Castling::WK > 0 { "K" } else { "" };
    castling_as_string += if p & Castling::WQ > 0 { "Q" } else { "" };
    castling_as_string += if p & Castling::BK > 0 { "k" } else { "" };
    castling_as_string += if p & Castling::BQ > 0 { "q" } else { "" };

    if castling_as_string.is_empty() {
        castling_as_string = String::from("-");
    }

    castling_as_string
}

// ===== Printing used for development purposes only =====

// This prints a bitboard (64-bit number) to the screen in an 8x8 grid.
#[allow(dead_code)]
pub fn bitboard(bitboard: Bitboard, mark_square: Option<u8>) {
    const SQUARE_OCCUPIED: char = '1';
    let mut ascii_board: AsciiBoard = [CHAR_ES; 64];
    put_character_on_square(bitboard, &mut ascii_board, SQUARE_OCCUPIED);
    to_console(&ascii_board, mark_square);
}

// Prints a given movelist to the screen.
#[allow(dead_code)]
pub fn movelist(ml: &MoveList) {
    for i in 0..ml.len() {
        move_data(ml.get_move(i), i);
    }
}

// Prints decoded move data to the screen.
#[allow(dead_code)]
pub fn move_data(m: Move, nr: u8) {
    println!(
        "{}. Move: {}{}{} capture: {}, promotion: {}, ep: {}, double: {}, castling: {}, score: {}",
        nr + 1,
        PIECE_CHAR_CAPS[m.piece()],
        SQUARE_NAME[m.from()],
        SQUARE_NAME[m.to()],
        PIECE_NAME[m.captured()],
        PIECE_NAME[m.promoted()],
        m.en_passant(),
        m.double_step(),
        m.castling(),
        m.get_sort_score(),
    );
}
