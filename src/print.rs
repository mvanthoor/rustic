use crate::board::Board;
use crate::defines::*;
use crate::movegen::MoveList;

type AsciiBoard = [char; NR_OF_SQUARES as usize];

const ASCII_EMPTY_SQUARE: char = '.';
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

const PIECE_CHAR: [&str; 7] = ["K", "Q", "R", "B", "N", "I", "-"];
const PIECE_NAME: [&str; 7] = ["King", "Queen", "Rook", "Bishop", "Knight", "Pawn", "None"];
// const COLOR_NAME: [&str; 2] = ["White", "black"];

pub fn engine_info() {
    println!();
    println!("Engine: {} {}", ENGINE, VERSION);
    println!("Author: {}", AUTHOR);
}

pub fn position(board: &Board, mark_square: Option<u8>) {
    let mut ascii_board: AsciiBoard = [ASCII_EMPTY_SQUARE; NR_OF_SQUARES as usize];
    bitboards_to_ascii(board, &mut ascii_board);
    to_console(&ascii_board, mark_square);
}

#[allow(dead_code)]
pub fn bitboard(bitboard: Bitboard, mark_square: Option<u8>) {
    const SQUARE_OCCUPIED: char = '1';
    let mut ascii_board: AsciiBoard = [ASCII_EMPTY_SQUARE; 64];
    put_character_on_square(bitboard, &mut ascii_board, SQUARE_OCCUPIED);
    to_console(&ascii_board, mark_square);
}

#[allow(dead_code)]
pub fn movelist(moves: &MoveList) {
    for m in moves.iter() {
        println!(
            "{}{}{} ({})",
            PIECE_CHAR[m.piece() as usize],
            SQUARE_NAME[m.from() as usize],
            SQUARE_NAME[m.to() as usize],
            PIECE_NAME[m.captured() as usize]
        );
    }
}

fn put_character_on_square(bitboard: Bitboard, ascii: &mut AsciiBoard, character: char) {
    for (i, square) in ascii.iter_mut().enumerate() {
        if (bitboard >> i) & 1 == 1 {
            *square = character;
        }
    }
}

fn bitboards_to_ascii(board: &Board, ascii: &mut AsciiBoard) {
    for (&bb_w, (board, &bb_b)) in board.bb_w.iter().zip(board.bb_b.iter().enumerate()) {
        match board {
            KING => {
                put_character_on_square(bb_w, ascii, CHAR_WK);
                put_character_on_square(bb_b, ascii, CHAR_BK);
            }
            QUEEN => {
                put_character_on_square(bb_w, ascii, CHAR_WQ);
                put_character_on_square(bb_b, ascii, CHAR_BQ);
            }
            ROOK => {
                put_character_on_square(bb_w, ascii, CHAR_WR);
                put_character_on_square(bb_b, ascii, CHAR_BR);
            }
            BISHOP => {
                put_character_on_square(bb_w, ascii, CHAR_WB);
                put_character_on_square(bb_b, ascii, CHAR_BB);
            }
            KNIGHT => {
                put_character_on_square(bb_w, ascii, CHAR_WN);
                put_character_on_square(bb_b, ascii, CHAR_BN);
            }
            PAWN => {
                put_character_on_square(bb_w, ascii, CHAR_WP);
                put_character_on_square(bb_b, ascii, CHAR_BP);
            }
            _ => (),
        }
    }
}

fn to_console(ascii_board: &AsciiBoard, mark_square: Option<u8>) {
    let coordinate_alpha: &str = "ABCDEFGH";
    let mut coordinate_digit = NR_OF_FILES;

    println!();
    for current_rank in ALL_RANKS.rev() {
        print!("{}   ", coordinate_digit);
        for current_file in ALL_FILES {
            let square = (current_rank * NR_OF_FILES + current_file) as usize;
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
