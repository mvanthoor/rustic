/**
 * fen.rs is the FEN-reader module. It takes an FEN-string, and reads it into the board.
 * Note that the FEN-reader will not accept FEN-strings with illegal syntax. If it encounters
 * illegal syntax, it'll abort the program because the board state would be undefined.
 * The reader *does* accept FEN-strings with board positions that are impossible in normal
 * chess, such as having two kings, 9 pawns, 10 queens, or both kings in check at the same time.
 * It is not the purpose of the FEN-reader to check position legality, so it doesn't.
*/
use crate::board::representation::Board;
use crate::defs::{
    BISHOP, BLACK, CASTLE_BK, CASTLE_BQ, CASTLE_WK, CASTLE_WQ, FILE_A, KING, KNIGHT, PAWN, QUEEN,
    RANK_8, ROOK, WHITE,
};

/** Definitions used by the FEN-reader */
const NR_OF_FEN_PARTS: usize = 6;
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const LETTERS: &str = "abcdefgh";
const EN_PASSANT_RANKS: &str = "36";
const WHITE_OR_BLACK: &str = "wb";
const CASTLE_RIGHTS: &str = "KQkq";
const SPLITTER: char = '/';
const DASH: char = '-';
const SPACE: char = ' ';
const MAX_FULL_MOVES: u16 = 9999;
type FenPartHandlers = fn(part: &str, board: &mut Board);

/** This is the only public function. It uses private functions to split up the parsing
 * of the FEN string. It works as follows:
 *      - First, the FEN-string is split up into parts.
 *      - Then, fen_parse loads all the private functions for parsing each part.
 *      - The number of parts is determined.
 *      - If the number of parts is correct, each part will be parsed by its own function.
*/
pub fn read(fen_string: &str, board: &mut Board) {
    let fen_parts: Vec<String> = fen_string.split(SPACE).map(|s| s.to_string()).collect();
    let fen_parse: [FenPartHandlers; 6] = [part_0, part_1, part_2, part_3, part_4, part_5];
    let length = fen_parts.len();

    if length == NR_OF_FEN_PARTS {
        board.reset();
        for (i, handle_part) in fen_parse.iter().enumerate() {
            handle_part(&fen_parts[i], board);
        }
    }
    assert!(
        length == NR_OF_FEN_PARTS,
        "FEN: Has {} parts instead of {}",
        length,
        NR_OF_FEN_PARTS
    );
}

/** Part 0: Parsing piece setup. Put each piece into its respective bitboard. */
fn part_0(part: &str, board: &mut Board) {
    const PART: u8 = 0;
    let mut rank = RANK_8 as u8;
    let mut file = FILE_A as u8;

    for c in part.chars() {
        let square = (rank * 8) + file;
        match c {
            'k' => board.bb_side[BLACK][KING] += 1u64 << square,
            'q' => board.bb_side[BLACK][QUEEN] += 1u64 << square,
            'r' => board.bb_side[BLACK][ROOK] += 1u64 << square,
            'b' => board.bb_side[BLACK][BISHOP] += 1u64 << square,
            'n' => board.bb_side[BLACK][KNIGHT] += 1u64 << square,
            'p' => board.bb_side[BLACK][PAWN] += 1u64 << square,
            'K' => board.bb_side[WHITE][KING] += 1u64 << square,
            'Q' => board.bb_side[WHITE][QUEEN] += 1u64 << square,
            'R' => board.bb_side[WHITE][ROOK] += 1u64 << square,
            'B' => board.bb_side[WHITE][BISHOP] += 1u64 << square,
            'N' => board.bb_side[WHITE][KNIGHT] += 1u64 << square,
            'P' => board.bb_side[WHITE][PAWN] += 1u64 << square,
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

/** Part 1: Parse color to move: White or Black */
fn part_1(part: &str, board: &mut Board) {
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

/** Part 2: Parse castling rights. */
fn part_2(part: &str, board: &mut Board) {
    const PART: u8 = 2;
    let length = part.len();
    let mut char_ok = 0;

    if length >= 1 && length <= 4 {
        // Accepts "-" for no castling rights in addition to leaving out letters.
        for c in part.chars() {
            if c == DASH {
                char_ok += 1
            }
            if CASTLE_RIGHTS.contains(c) {
                char_ok += 1;
                match c {
                    'K' => board.game_state.castling += CASTLE_WK,
                    'Q' => board.game_state.castling += CASTLE_WQ,
                    'k' => board.game_state.castling += CASTLE_BK,
                    'q' => board.game_state.castling += CASTLE_BQ,
                    _ => (),
                }
            }
        }
    }
    assert!(length >= 1, "FEN {}: Castling rights: {}", PART, part);
    assert_eq!(char_ok, length, "FEN {}: Castling rights: {}", PART, part);
}

/**
 * Parse en-passant square. This is either an algebraic square, or a dash.
 * If a square is found, it'll be converted into a square number and put into
 * the board representation. Otherwise, None will be entered.
 */
fn part_3(part: &str, board: &mut Board) {
    const PART: u8 = 3;
    let length = part.len();
    let mut char_ok = 0;

    if length == 1 {
        if let Some(x) = part.chars().next() {
            if x == DASH {
                char_ok += 1
            }
        }
    }

    if length == 2 {
        const ASCII_VALUE_OF_SMALL_A: u8 = 97;
        const ASCII_VALUE_OF_1: u8 = 49;
        let mut file = 0;
        let mut rank = 0;
        for (char_nr, c) in part.chars().enumerate() {
            if char_nr == 0 && LETTERS.contains(c) {
                file = (c as u8) - ASCII_VALUE_OF_SMALL_A;
                char_ok += 1;
            }
            if char_nr == 1 && EN_PASSANT_RANKS.contains(c) {
                rank = (c as u8) - ASCII_VALUE_OF_1;
                char_ok += 1;
            }
        }
        if char_ok == length {
            let square_nr = (rank * 8) + file;
            board.game_state.en_passant = Some(square_nr);
        }
    }
    assert_eq!(char_ok, length, "FEN {}: En Passant Target: {}", PART, part);
}

/** Half-move clock: parse number of moves since last capture or pawn push. */
fn part_4(part: &str, board: &mut Board) {
    const PART: u8 = 4;
    let length = part.len();
    let mut is_ok = false;

    if length == 1 || length == 2 {
        if let Ok(x) = part.parse::<u8>() {
            if x <= 50 {
                board.game_state.halfmove_clock = x;
                is_ok = true;
            }
        }
    }
    assert!(is_ok, "FEN {}: 50-move count: {}", PART, part);
}

//** Parse number of moves made in the game. */
fn part_5(part: &str, board: &mut Board) {
    const PART: u8 = 5;
    let length = part.len();
    let mut is_ok = false;

    if length >= 1 || length <= 4 {
        if let Ok(x) = part.parse::<u16>() {
            if x <= MAX_FULL_MOVES {
                board.game_state.fullmove_number = x;
                is_ok = true;
            }
        }
    }
    assert!(is_ok, "FEN {}: Full move count: {}", PART, part);
}
