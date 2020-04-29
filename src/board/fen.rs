// fen.rs reads an FEN-string and converts it into a board position.
// If the procedure fails, the original position is not changed.
use crate::board::{representation::Board, Files, Pieces, Ranks, Squares};
use crate::defs::{Castling, Square, BLACK, MAX_GAME_MOVES, WHITE};
use crate::parse;
use if_chain::if_chain;
use std::ops::RangeInclusive;

// Define errors
pub const ERR_FEN_PARTS: [&str; NR_OF_FEN_PARTS as usize + 1] = [
    "Must have six (6) parts.",
    "Pieces and squares.",
    "Color selection.",
    "Castling permissions.",
    "En-passant square.",
    "Half-move clock.",
    "Full-move number.",
];

/** Definitions used by the FEN-reader */
const NR_OF_FEN_PARTS: usize = 6;
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const EP_SQUARES_WHITE: RangeInclusive<Square> = Squares::A3..=Squares::H3;
const EP_SQUARES_BLACK: RangeInclusive<Square> = Squares::A6..=Squares::H6;
const WHITE_OR_BLACK: &str = "wb";
const CASTLE_RIGHTS: &str = "KQkq-";
const SPLITTER: char = '/';
const DASH: char = '-';
const SPACE: char = ' ';
type FenPartParser = fn(board: &mut Board, part: &str) -> bool;

// This function splits the FEN-string into parts,
// and then runs the parsing function for each part.
pub fn read(board: &mut Board, fen_string: &str) -> Result<(), u8> {
    let fen_parts: Vec<String> = fen_string.split(SPACE).map(|s| s.to_string()).collect();
    let fen_parsers: [FenPartParser; 6] = [pieces, color, castling, ep, hmc, fmn];
    let length = fen_parts.len();
    let mut result: Result<(), u8> = Err(0);

    if length == NR_OF_FEN_PARTS {
        // Clone the incoming board so we don't need to create one from scratch.
        let mut try_board = board.clone();
        try_board.reset();

        // Try setup on the try_board, so we don't ruin our existing setup.
        for (i, parser) in fen_parsers.iter().enumerate() {
            if parser(&mut try_board, &fen_parts[i]) {
                result = Ok(());
            } else {
                result = Err(i as u8 + 1);
                break;
            }
        }

        // If setup of the pieces is successful, initialize and replace.
        if result == Ok(()) {
            try_board.init();
            *board = try_board;
        }
    }
    result
}

// Part 1: Parsing piece setup. Put each piece into its respective bitboard.
fn pieces(board: &mut Board, part: &str) -> bool {
    let mut rank = Ranks::R8 as u8;
    let mut file = Files::A as u8;
    let mut result = true;

    for c in part.chars() {
        let square = (rank * 8) + file;
        match c {
            'k' => board.bb_side[BLACK][Pieces::KING] |= 1u64 << square,
            'q' => board.bb_side[BLACK][Pieces::QUEEN] |= 1u64 << square,
            'r' => board.bb_side[BLACK][Pieces::ROOK] |= 1u64 << square,
            'b' => board.bb_side[BLACK][Pieces::BISHOP] |= 1u64 << square,
            'n' => board.bb_side[BLACK][Pieces::KNIGHT] |= 1u64 << square,
            'p' => board.bb_side[BLACK][Pieces::PAWN] |= 1u64 << square,
            'K' => board.bb_side[WHITE][Pieces::KING] |= 1u64 << square,
            'Q' => board.bb_side[WHITE][Pieces::QUEEN] |= 1u64 << square,
            'R' => board.bb_side[WHITE][Pieces::ROOK] |= 1u64 << square,
            'B' => board.bb_side[WHITE][Pieces::BISHOP] |= 1u64 << square,
            'N' => board.bb_side[WHITE][Pieces::KNIGHT] |= 1u64 << square,
            'P' => board.bb_side[WHITE][Pieces::PAWN] |= 1u64 << square,
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
            _ => result = false,
        }

        if LIST_OF_PIECES.contains(c) {
            file += 1;
        }

        if !result {
            break;
        }
    }
    result
}

// Part 2: Parse color to move: White or Black
fn color(board: &mut Board, part: &str) -> bool {
    let mut result = false;

    if_chain! {
        if part.len() == 1;
        if let Some(x) = part.chars().next();
        if WHITE_OR_BLACK.contains(x);
        then {
            match x {
                'w' => board.game_state.active_color = WHITE as u8,
                'b' => board.game_state.active_color = BLACK as u8,
                _ => (),
            }
            result = true;
        }
    }
    result
}

// Part 3: Parse castling rights.
fn castling(board: &mut Board, part: &str) -> bool {
    let length = part.len();
    let mut char_ok = 0;

    if length >= 1 && length <= 4 {
        // Accepts "-" for no castling rights in addition to leaving out letters.
        for c in part.chars() {
            if CASTLE_RIGHTS.contains(c) {
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
    (length >= 1) && (char_ok == length)
}

// Part 4: Parse the en passant square
fn ep(board: &mut Board, part: &str) -> bool {
    let length = part.len();
    let mut char_ok = 0;

    if_chain! {
        if length == 1;
        if let Some(x) = part.chars().next();
        if x == DASH;
        then {
            char_ok += 1
        }
    }

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
