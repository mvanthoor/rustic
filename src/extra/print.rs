use crate::{
    board::{
        defs::{Pieces, RangeOf, PIECE_NAME, SQUARE_NAME},
        Board,
    },
    defs::{Bitboard, Castling, NrOf, Sides, Square},
    movegen::{defs::Move, magics::Magics},
};

type AsciiBoard = [char; NrOf::SQUARES];

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
const PIECE_CHAR: [&str; 7] = ["K", "Q", "R", "B", "N", "", "_"];

// Prints the current position to the screen.
#[allow(dead_code)]
pub fn position(board: &Board, mark_square: Option<u8>) {
    let mut ascii_board: AsciiBoard = [ASCII_EMPTY_SQUARE; NrOf::SQUARES];
    bitboards_to_ascii(board, &mut ascii_board);
    to_console(&ascii_board, mark_square);
    metadata(board);
}

// This prints a bitboard (64-bit number) to the screen in an 8x8 grid.
#[allow(dead_code)]
pub fn bitboard(bitboard: Bitboard, mark_square: Option<u8>) {
    const SQUARE_OCCUPIED: char = '1';
    let mut ascii_board: AsciiBoard = [ASCII_EMPTY_SQUARE; 64];
    put_character_on_square(bitboard, &mut ascii_board, SQUARE_OCCUPIED);
    to_console(&ascii_board, mark_square);
}

// Prints a given movelist to the screen.
#[allow(dead_code)]
pub fn movelist(moves: &[Move]) {
    for m in moves.iter() {
        move_data(*m);
    }
}

// Prints decoded move data to the screen.
#[allow(dead_code)]
pub fn move_data(m: Move) {
    println!(
        "Move: {}{}{} capture: {}, promotion: {}, ep: {}, double: {}, castling: {}",
        PIECE_CHAR[m.piece()],
        SQUARE_NAME[m.from()],
        SQUARE_NAME[m.to()],
        PIECE_NAME[m.captured()],
        PIECE_NAME[m.promoted()],
        m.en_passant(),
        m.double_step(),
        m.castling(),
    );
}

// This function prints a found magic number and its stats.
#[allow(dead_code)]
pub fn found_magic(square: Square, m: Magics, offset: u64, end: u64, attempts: u64) {
    println!(
        "Magic found for {}: {:24}u64 (offset: {:6} end: {:6}, attempts: {})",
        SQUARE_NAME[square], m.magic, offset, end, attempts
    );
}

#[allow(dead_code)]
pub fn horizontal_line(c: char, length: u8) {
    for _ in 0..length {
        print!("{}", c);
    }
    println!();
}

// Create a printable ASCII-board out of bitboards.
#[allow(dead_code)]
fn bitboards_to_ascii(board: &Board, ascii_board: &mut AsciiBoard) {
    let bb_w = board.bb_side[Sides::WHITE];
    let bb_b = board.bb_side[Sides::BLACK];

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
#[allow(dead_code)]
fn put_character_on_square(bitboard: Bitboard, ascii_board: &mut AsciiBoard, character: char) {
    for (i, square) in ascii_board.iter_mut().enumerate() {
        if (bitboard >> i) & 1 == 1 {
            *square = character;
        }
    }
}

// Print the generated ASCII-board to the console. Optionally mark one square.
#[allow(dead_code)]
fn to_console(ascii_board: &AsciiBoard, mark_square: Option<u8>) {
    let coordinate_alpha: &str = "ABCDEFGH";
    let mut coordinate_digit = NrOf::FILES;

    println!();
    for current_rank in RangeOf::RANKS.rev() {
        print!("{}   ", coordinate_digit);
        for current_file in RangeOf::FILES {
            let square = (current_rank * NrOf::FILES + current_file) as usize;
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
#[allow(dead_code)]
fn metadata(board: &Board) {
    let castling = castling_as_string(board.game_state.castling);
    let en_passant = if let Some(ep) = board.game_state.en_passant {
        SQUARE_NAME[ep as usize]
    } else {
        "-"
    };
    let active_color = if (board.game_state.active_color as usize) == Sides::WHITE {
        "White"
    } else {
        "Black"
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
#[allow(dead_code)]
fn castling_as_string(permissions: u8) -> String {
    let mut castling_as_string: String = String::from("");
    let p = permissions;

    castling_as_string += if p & Castling::WK > 0 { "K" } else { "" };
    castling_as_string += if p & Castling::WQ > 0 { "Q" } else { "" };
    castling_as_string += if p & Castling::BK > 0 { "k" } else { "" };
    castling_as_string += if p & Castling::BQ > 0 { "q" } else { "" };

    castling_as_string
}
