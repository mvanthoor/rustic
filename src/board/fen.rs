// fen.rs reads an FEN-string and converts it into a board position.
// If the procedure fails, the original position is not changed.
use super::{
    defs::{Files, Pieces, Ranks, Squares},
    Board,
};
use crate::{
    defs::{Castling, Sides, Square, FEN_START_POSITION, MAX_GAME_MOVES},
    misc::parse,
};
use if_chain::if_chain;
use std::ops::RangeInclusive;

/** Definitions used by the FEN-reader */
const NR_OF_FEN_PARTS: usize = 6;
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const EP_SQUARES_WHITE: RangeInclusive<Square> = Squares::A3..=Squares::H3;
const EP_SQUARES_BLACK: RangeInclusive<Square> = Squares::A6..=Squares::H6;
const WHITE_OR_BLACK: &str = "wb";
const CASTLING_RIGHTS: &str = "KQkq-";
const SPLITTER: char = '/';
const DASH: char = '-';
const SPACE: char = ' ';

type FenPartParser = fn(board: &mut Board, part: &str) -> bool;
type FenResult = Result<(), u8>;

// Define errors
#[allow(dead_code)]
pub const ERR_FEN_PARTS: [&str; NR_OF_FEN_PARTS + 1] = [
    "Must have six (6) parts",
    "Pieces and squares",
    "Color selection",
    "Castling permissions",
    "En-passant square",
    "Half-move clock",
    "Full-move number",
];

impl Board {
    // This function reads a provided FEN-string or uses the default position.
    pub fn fen_read(&mut self, fen_string: Option<&str>) -> FenResult {
        // Split the string into parts. There should be 6 parts.
        let fen_parts: Vec<String> = match fen_string {
            Some(f) => f,
            None => FEN_START_POSITION,
        }
        .split(SPACE)
        .map(|s| s.to_string())
        .collect();

        // Check the number of fen parts.
        let nr_of_parts_ok = fen_parts.len() == NR_OF_FEN_PARTS;

        // Set the initial result.
        let mut result: FenResult = if nr_of_parts_ok { Ok(()) } else { Err(0) };

        if nr_of_parts_ok {
            // Create an array of function pointers; one parsing function per part.
            let fen_parsers: [FenPartParser; 6] = [pieces, color, castling, ep, hmc, fmn];

            // Create a new board so we don't destroy the original.
            let mut new_board = self.clone();
            new_board.reset();

            // Parse all the parts and check if each one succeeds.
            let mut i: usize = 0;
            while i < NR_OF_FEN_PARTS && result == Ok(()) {
                let parser = &fen_parsers[i];
                let part = &fen_parts[i];
                let part_ok = parser(&mut new_board, part);
                result = if part_ok { Ok(()) } else { Err(i as u8 + 1) };
                i += 1;
            }

            // Replace original board with new one if setup was successful.
            if result == Ok(()) {
                new_board.init();
                *self = new_board;
            }
        }

        result
    }
}

// ===== Private functions =====

// Part 1: Parsing piece setup. Put each piece into its respective bitboard.
fn pieces(board: &mut Board, part: &str) -> bool {
    let mut rank = Ranks::R8 as u8;
    let mut file = Files::A as u8;

    // Assume parsing succeeds.
    let mut result = true;

    // Parse each character; it should be a piece, square count, or splitter.
    for c in part.chars() {
        let square = (rank * 8) + file;
        match c {
            'k' => board.bb_pieces[Sides::BLACK][Pieces::KING] |= 1u64 << square,
            'q' => board.bb_pieces[Sides::BLACK][Pieces::QUEEN] |= 1u64 << square,
            'r' => board.bb_pieces[Sides::BLACK][Pieces::ROOK] |= 1u64 << square,
            'b' => board.bb_pieces[Sides::BLACK][Pieces::BISHOP] |= 1u64 << square,
            'n' => board.bb_pieces[Sides::BLACK][Pieces::KNIGHT] |= 1u64 << square,
            'p' => board.bb_pieces[Sides::BLACK][Pieces::PAWN] |= 1u64 << square,
            'K' => board.bb_pieces[Sides::WHITE][Pieces::KING] |= 1u64 << square,
            'Q' => board.bb_pieces[Sides::WHITE][Pieces::QUEEN] |= 1u64 << square,
            'R' => board.bb_pieces[Sides::WHITE][Pieces::ROOK] |= 1u64 << square,
            'B' => board.bb_pieces[Sides::WHITE][Pieces::BISHOP] |= 1u64 << square,
            'N' => board.bb_pieces[Sides::WHITE][Pieces::KNIGHT] |= 1u64 << square,
            'P' => board.bb_pieces[Sides::WHITE][Pieces::PAWN] |= 1u64 << square,
            '1'..='8' => {
                if let Some(x) = c.to_digit(10) {
                    file += x as u8;
                }
            }
            SPLITTER => {
                result = file == 8;
                rank -= 1;
                file = 0;
            }
            // Unknown character: result becomes false.
            _ => result = false,
        }

        // If piece found, advance to the next file.
        if LIST_OF_PIECES.contains(c) {
            file += 1;
        }

        // As soon as something is wrong, stop parsing.
        if !result {
            break;
        }
    }

    result
}

// Part 2: Parse color to move: White or Black
fn color(board: &mut Board, part: &str) -> bool {
    // Assume parsing fails.
    let mut result = false;

    // Length should be 1, and the character should be 'w' or 'b'.
    if_chain! {
        if part.len() == 1;
        if let Some(x) = part.chars().next();
        if WHITE_OR_BLACK.contains(x);
        then {
            match x {
                'w' => board.game_state.active_color = Sides::WHITE as u8,
                'b' => board.game_state.active_color = Sides::BLACK as u8,
                _ => (),
            }

            // If everything is correct, set the result to true;
            result = true;
        }
    }

    result
}

// Part 3: Parse castling rights.
fn castling(board: &mut Board, part: &str) -> bool {
    let length = part.len();
    let mut char_ok = 0;

    // There should be 1 to 4 castling rights. If no player has castling
    // rights, the character is '-'.
    if length >= 1 && length <= 4 {
        // Accepts "-" for no castling rights in addition to leaving out letters.
        for c in part.chars() {
            if CASTLING_RIGHTS.contains(c) {
                // Count correct characters
                char_ok += 1;
                match c {
                    'K' => board.game_state.castling |= Castling::WK,
                    'Q' => board.game_state.castling |= Castling::WQ,
                    'k' => board.game_state.castling |= Castling::BK,
                    'q' => board.game_state.castling |= Castling::BQ,
                    _ => (),
                }
            }
        }
    }

    // Counted correct characters should be at least 1, and equal to the
    // length of the part.
    (length >= 1) && (char_ok == length)
}

// Part 4: Parse the en passant square
fn ep(board: &mut Board, part: &str) -> bool {
    let length = part.len();
    let mut char_ok = 0;

    // No en-passant square if length is 1. The character should be a DASH.
    if_chain! {
        if length == 1;
        if let Some(x) = part.chars().next();
        if x == DASH;
        then {
            char_ok += 1
        }
    }

    // If length is 2, try to parse the part to a square number.
    if length == 2 {
        let square = parse::algebraic_square_to_number(part);
        match square {
            Ok(s) if EP_SQUARES_WHITE.contains(&s) || EP_SQUARES_BLACK.contains(&s) => {
                board.game_state.en_passant = Some(s as u8);
                char_ok += 2;
            }
            Ok(_) | Err(_) => (),
        }
    }

    // The length of this part should either be 1 or 2, and the counted
    // correct characters should be equal to the part length.
    (length == 1 || length == 2) && (length == char_ok)
}

// Part 5: Half-move clock: parse number of moves since last capture or pawn push.
fn hmc(board: &mut Board, part: &str) -> bool {
    let length = part.len();
    let mut result = false;

    if_chain! {
        if length == 1 || length == 2;
        if let Ok(x) = part.parse::<u8>();
        if x <= 50;
        then {
            board.game_state.halfmove_clock = x;
            result = true;
        }
    }

    result
}

// Part 6: Parse full move number.
fn fmn(board: &mut Board, part: &str) -> bool {
    let length = part.len();
    let mut result = false;

    if_chain! {
        if length >= 1 || length <= 4;
        if let Ok(x) = part.parse::<u16>();
        if x <= (MAX_GAME_MOVES as u16);
        then {
            board.game_state.fullmove_number = x;
            result = true;
        }
    }

    result
}
