// TODO: Update comment
use crate::board::representation::Board;
use crate::defs::{
    BISHOP, BLACK, CASTLE_BK, CASTLE_BQ, CASTLE_WK, CASTLE_WQ, FILE_A, KING, KNIGHT,
    MAX_GAME_MOVES, PAWN, QUEEN, RANK_8, ROOK, WHITE,
};
use crate::parse;
use if_chain::if_chain;
use std::ops::RangeInclusive;

/** Definitions used by the FEN-reader */
const NR_OF_FEN_PARTS: usize = 6;
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const EP_SQUARES_WHITE: RangeInclusive<u8> = 16..=23;
const EP_SQUARES_BLACK: RangeInclusive<u8> = 40..=47;
const WHITE_OR_BLACK: &str = "wb";
const CASTLE_RIGHTS: &str = "KQkq-";
const SPLITTER: char = '/';
const DASH: char = '-';
const SPACE: char = ' ';
type FenPartParser = fn(part: &str, board: &mut Board);

// This function splits the FEN-string into parts,
// and then runs the parsing function for each part.
pub fn read(board: &mut Board, fen_string: &str) {
    let fen_parts: Vec<String> = fen_string.split(SPACE).map(|s| s.to_string()).collect();
    let fen_parsers: [FenPartParser; 6] = [pieces, color, castling, ep, hmc, fmn];
    let length = fen_parts.len();

    if length == NR_OF_FEN_PARTS {
        board.reset();
        for (i, parser) in fen_parsers.iter().enumerate() {
            parser(&fen_parts[i], board);
        }
    }

    assert!(
        length == NR_OF_FEN_PARTS,
        "FEN: Has {} parts instead of {}",
        length,
        NR_OF_FEN_PARTS
    );
}

/** Parsing piece setup. Put each piece into its respective bitboard. */
fn pieces(part: &str, board: &mut Board) {
    const PART: u8 = 0;
    let mut rank = RANK_8 as u8;
    let mut file = FILE_A as u8;

    for c in part.chars() {
        let square = (rank * 8) + file;
        match c {
            'k' => board.bb_side[BLACK][KING] |= 1u64 << square,
            'q' => board.bb_side[BLACK][QUEEN] |= 1u64 << square,
            'r' => board.bb_side[BLACK][ROOK] |= 1u64 << square,
            'b' => board.bb_side[BLACK][BISHOP] |= 1u64 << square,
            'n' => board.bb_side[BLACK][KNIGHT] |= 1u64 << square,
            'p' => board.bb_side[BLACK][PAWN] |= 1u64 << square,
            'K' => board.bb_side[WHITE][KING] |= 1u64 << square,
            'Q' => board.bb_side[WHITE][QUEEN] |= 1u64 << square,
            'R' => board.bb_side[WHITE][ROOK] |= 1u64 << square,
            'B' => board.bb_side[WHITE][BISHOP] |= 1u64 << square,
            'N' => board.bb_side[WHITE][KNIGHT] |= 1u64 << square,
            'P' => board.bb_side[WHITE][PAWN] |= 1u64 << square,
            '1'..='8' => {
                if let Some(x) = c.to_digit(10) {
                    file += x as u8;
                }
            }
            SPLITTER => {
                assert!(file == 8, "FEN {}: Counting incorrect: {}", PART, part);
                rank -= 1;
                file = 0;
            }
            _ => panic!("FEN {}: Illegal character found: {}", PART, part),
        }
        if LIST_OF_PIECES.contains(c) {
            file += 1;
        }
    }
}

/** Parse color to move: White or Black */
fn color(part: &str, board: &mut Board) {
    const PART: u8 = 1;
    let mut step = if part.len() == 1 { 1 } else { 0 };

    if step == 1 {
        if let Some(x) = part.chars().next() {
            step += if WHITE_OR_BLACK.contains(x) { 1 } else { 0 };
            match x {
                'w' => board.game_state.active_color = WHITE as u8,
                'b' => board.game_state.active_color = BLACK as u8,
                _ => (),
            }
        }
    }
    assert_eq!(step, 2, "FEN {}: Must be 'w' or 'b'. {}", PART, part);
}

/** Parse castling rights. */
fn castling(part: &str, board: &mut Board) {
    const PART: u8 = 2;
    let length = part.len();
    let mut char_ok = 0;

    if length >= 1 && length <= 4 {
        // Accepts "-" for no castling rights in addition to leaving out letters.
        for c in part.chars() {
            if CASTLE_RIGHTS.contains(c) {
                char_ok += 1;
                match c {
                    'K' => board.game_state.castling |= CASTLE_WK,
                    'Q' => board.game_state.castling |= CASTLE_WQ,
                    'k' => board.game_state.castling |= CASTLE_BK,
                    'q' => board.game_state.castling |= CASTLE_BQ,
                    _ => (),
                }
            }
        }
    }
    assert!(length >= 1, "FEN {}: Castling rights: {}", PART, part);
    assert_eq!(char_ok, length, "FEN {}: Castling rights: {}", PART, part);
}

// Parse the en passant square.
fn ep(part: &str, board: &mut Board) {
    const PART: u8 = 3;
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
                board.game_state.en_passant = Some(s);
                char_ok += 2;
            }
            Ok(_) | Err(_) => (),
        }
    }
    assert_eq!(char_ok, length, "FEN {}: En Passant Target: {}", PART, part);
}

/** Half-move clock: parse number of moves since last capture or pawn push. */
fn hmc(part: &str, board: &mut Board) {
    const PART: u8 = 4;
    let length = part.len();
    let mut is_ok = false;

    if_chain! {
        if length == 1 || length == 2;
        if let Ok(x) = part.parse::<u8>();
        if x <= 50;
        then {
            board.game_state.halfmove_clock = x;
            is_ok = true;
        }
    }
    assert!(is_ok, "FEN {}: 50-move count: {}", PART, part);
}

//** Parse full move number. */
fn fmn(part: &str, board: &mut Board) {
    const PART: u8 = 5;
    let length = part.len();
    let mut is_ok = false;

    if_chain! {
        if length >= 1 || length <= 4;
        if let Ok(x) = part.parse::<u16>();
        if x <= (MAX_GAME_MOVES as u16);
        then {
            board.game_state.fullmove_number = x;
            is_ok = true;
        }
    }
    assert!(is_ok, "FEN {}: Full move count: {}", PART, part);
}
