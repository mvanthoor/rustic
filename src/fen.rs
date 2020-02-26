use crate::board::Board;
use crate::defines::{
    BISHOP, BLACK, CASTLE_BK, CASTLE_BQ, CASTLE_WK, CASTLE_WQ, FILE_A, KING, KNIGHT, PAWN, QUEEN,
    RANK_8, ROOK, WHITE,
};

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

pub fn read(fen_string: &str, board: &mut Board) {
    let fen_parts: Vec<String> = fen_string.split(SPACE).map(|s| s.to_string()).collect();
    let fen_parse: [FenPartHandlers; 6] = [part_0, part_1, part_2, part_3, part_4, part_5];
    let length = fen_parts.len();

    if length == NR_OF_FEN_PARTS {
        board.reset();
        for (i, handle_part) in fen_parse.iter().enumerate() {
            handle_part(&fen_parts[i], board);
        }
        board.create_piece_bitboards();
    }
    assert!(
        length == NR_OF_FEN_PARTS,
        "FEN: Has {} parts instead of {}",
        length,
        NR_OF_FEN_PARTS
    );
}

fn part_0(part: &str, board: &mut Board) {
    const PART: u8 = 0;
    let mut rank = RANK_8 as u8;
    let mut file = FILE_A as u8;

    for c in part.chars() {
        let square = (rank * 8) + file;
        match c {
            'k' => board.bb_b[KING] += 1u64 << square,
            'q' => board.bb_b[QUEEN] += 1u64 << square,
            'r' => board.bb_b[ROOK] += 1u64 << square,
            'b' => board.bb_b[BISHOP] += 1u64 << square,
            'n' => board.bb_b[KNIGHT] += 1u64 << square,
            'p' => board.bb_b[PAWN] += 1u64 << square,
            'K' => board.bb_w[KING] += 1u64 << square,
            'Q' => board.bb_w[QUEEN] += 1u64 << square,
            'R' => board.bb_w[ROOK] += 1u64 << square,
            'B' => board.bb_w[BISHOP] += 1u64 << square,
            'N' => board.bb_w[KNIGHT] += 1u64 << square,
            'P' => board.bb_w[PAWN] += 1u64 << square,
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

fn part_1(part: &str, board: &mut Board) {
    const PART: u8 = 1;
    let mut step = if part.len() == 1 { 1 } else { 0 };

    if step == 1 {
        if let Some(x) = part.chars().next() {
            step += if WHITE_OR_BLACK.contains(x) { 1 } else { 0 };
            match x {
                'w' => board.active_color = WHITE as u8,
                'b' => board.active_color = BLACK as u8,
                _ => (),
            }
        }
    }
    assert_eq!(step, 2, "FEN {}: Must be 'w' or 'b'. {}", PART, part);
}

fn part_2(part: &str, board: &mut Board) {
    const PART: u8 = 2;
    let length = part.len();
    let mut char_ok = 0;

    if length == 1 {
        if let Some(x) = part.chars().next() {
            if x == DASH || CASTLE_RIGHTS.contains(x) {
                char_ok += 1
            }
        }
    }

    if length > 1 && length <= 4 {
        for c in part.chars() {
            if CASTLE_RIGHTS.contains(c) {
                char_ok += 1;
                match c {
                    'K' => board.castling += CASTLE_WK,
                    'Q' => board.castling += CASTLE_WQ,
                    'k' => board.castling += CASTLE_BK,
                    'q' => board.castling += CASTLE_BQ,
                    _ => (),
                }
            }
        }
    }
    assert_eq!(char_ok, length, "FEN {}: Castling rights: {}", PART, part);
}

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
        if file != 0 && rank != 0 {
            let square_nr = (rank * 8) + file;
            board.en_passant = Some(square_nr);
        }
    }
    assert_eq!(char_ok, length, "FEN {}: En Passant Target: {}", PART, part);
}

fn part_4(part: &str, board: &mut Board) {
    const PART: u8 = 4;
    let length = part.len();
    let mut is_ok = false;

    if length == 1 || length == 2 {
        if let Ok(x) = part.parse::<u8>() {
            if x <= 50 {
                board.halfmove_clock = x;
                is_ok = true;
            }
        }
    }
    assert!(is_ok, "FEN {}: 50-move count: {}", PART, part);
}

fn part_5(part: &str, board: &mut Board) {
    const PART: u8 = 5;
    let length = part.len();
    let mut is_ok = false;

    if length >= 1 || length <= 4 {
        if let Ok(x) = part.parse::<u16>() {
            if x <= MAX_FULL_MOVES {
                board.fullmove_number = x;
                is_ok = true;
            }
        }
    }
    assert!(is_ok, "FEN {}: Full move count: {}", PART, part);
}
