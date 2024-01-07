// fen.rs reads an FEN-string and converts it into a board position.
// If the procedure fails, the original position is not changed. Note that
// checking position legality is not the responsibility of this module. It
// is perfectly possible to set up a position with two white kings, both
// kings in check at the same time, or with black in check but white to
// move.

use crate::{
    board::defs::{Files, Pieces, Ranks, Squares, BB_SQUARES},
    board::Board,
    defs::{Castling, Sides, Square, FEN_START_POSITION, MAX_GAME_MOVES, MAX_MOVE_RULE},
    misc::parse,
};
use if_chain::if_chain;
use std::{
    fmt::{self, Display},
    ops::RangeInclusive,
};

/** Definitions used by the FEN-reader */
const CORRECT_FEN_LENGTH: usize = 6;
const SHORT_FEN_LENGTH: usize = 4;
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const EP_SQUARES_WHITE: RangeInclusive<Square> = Squares::A3..=Squares::H3;
const EP_SQUARES_BLACK: RangeInclusive<Square> = Squares::A6..=Squares::H6;
const WHITE_OR_BLACK: &str = "wb";
const SPLITTER: char = '/';
const DASH: char = '-';
const EM_DASH: char = '–';
const SPACE: char = ' ';

#[derive(Debug)]
pub enum FenError {
    IncorrectLength,
    Part1,
    Part2,
    Part3,
    Part4,
    Part5,
    Part6,
}

impl Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            Self::IncorrectLength => "Error in FEN string: Must be 6 parts",
            Self::Part1 => "Error in FEN Part 1: Pieces or squares",
            Self::Part2 => "Error in FEN Part 2: Colors",
            Self::Part3 => "Error in FEN Part 3: Castling rights",
            Self::Part4 => "Error in FEN Part 4: En passant field",
            Self::Part5 => "Error in FEN Part 5: Half-move clock",
            Self::Part6 => "Error in FEN Part 6: Full-move number",
        };
        write!(f, "{error}")
    }
}

pub type FenResult = Result<(), FenError>;
type FenPartParser = fn(board: &mut Board, part: &str) -> FenResult;

impl Board {
    // This function reads a provided FEN-string or uses the default position.
    pub fn fen_setup(&mut self, fen_string: Option<&str>) -> FenResult {
        // Split the string into parts. There should be 6 parts.
        let mut fen_parts = split_fen_string(fen_string);

        // However, if its a short fen, extend it with the missing two parts.
        if fen_parts.len() == SHORT_FEN_LENGTH {
            fen_parts.append(&mut vec![String::from("0"), String::from("1")]);
        }

        if fen_parts.len() != CORRECT_FEN_LENGTH {
            return Err(FenError::IncorrectLength);
        }

        let fen_parsers = create_part_parsers();

        // Create a new board so we don't destroy the original if the
        // fen-string happens to be incorrect.
        let mut new_board = self.clone();
        new_board.reset();

        // Parse all the parts and check if each one succeeds. If not,
        // immediately return with the error of the offending part.
        for (parser, part) in fen_parsers.iter().zip(fen_parts.iter()) {
            parser(&mut new_board, part)?;
        }

        // Replace original board with new one if setup was successful.
        new_board.init();
        *self = new_board;

        Ok(())
    }
}

// ===== Private functions =====

fn split_fen_string(fen_string: Option<&str>) -> Vec<String> {
    match fen_string {
        Some(fen) => fen,
        None => FEN_START_POSITION,
    }
    .replace(EM_DASH, DASH.encode_utf8(&mut [0; 4]))
    .split(SPACE)
    .map(String::from)
    .collect()
}

fn create_part_parsers() -> [FenPartParser; CORRECT_FEN_LENGTH] {
    [
        pieces,
        color,
        castling,
        en_passant,
        half_move_clock,
        full_move_number,
    ]
}

// Part 1: Parsing piece setup. Put each piece into its respective bitboard.
fn pieces(board: &mut Board, part: &str) -> FenResult {
    let mut rank = Ranks::R8 as u8;
    let mut file = Files::A as u8;

    // Parse each character; it should be a piece, square count, or splitter.
    for c in part.chars() {
        let square = ((rank * 8) + file) as usize;
        match c {
            'k' => board.bb_pieces[Sides::BLACK][Pieces::KING] |= BB_SQUARES[square],
            'q' => board.bb_pieces[Sides::BLACK][Pieces::QUEEN] |= BB_SQUARES[square],
            'r' => board.bb_pieces[Sides::BLACK][Pieces::ROOK] |= BB_SQUARES[square],
            'b' => board.bb_pieces[Sides::BLACK][Pieces::BISHOP] |= BB_SQUARES[square],
            'n' => board.bb_pieces[Sides::BLACK][Pieces::KNIGHT] |= BB_SQUARES[square],
            'p' => board.bb_pieces[Sides::BLACK][Pieces::PAWN] |= BB_SQUARES[square],
            'K' => board.bb_pieces[Sides::WHITE][Pieces::KING] |= BB_SQUARES[square],
            'Q' => board.bb_pieces[Sides::WHITE][Pieces::QUEEN] |= BB_SQUARES[square],
            'R' => board.bb_pieces[Sides::WHITE][Pieces::ROOK] |= BB_SQUARES[square],
            'B' => board.bb_pieces[Sides::WHITE][Pieces::BISHOP] |= BB_SQUARES[square],
            'N' => board.bb_pieces[Sides::WHITE][Pieces::KNIGHT] |= BB_SQUARES[square],
            'P' => board.bb_pieces[Sides::WHITE][Pieces::PAWN] |= BB_SQUARES[square],
            '1'..='8' => {
                if let Some(x) = c.to_digit(10) {
                    file += x as u8;
                }
            }
            SPLITTER => {
                if file != 8 {
                    return Err(FenError::Part1);
                }
                rank -= 1;
                file = 0;
            }
            _ => return Err(FenError::Part1),
        }

        // If a piece found, advance to the next file. (So we don't need to
        // do this in each piece's match arm above.)
        if LIST_OF_PIECES.contains(c) {
            file += 1;
        }
    }

    Ok(())
}

// Part 2: Parse color to move: White or Black
fn color(board: &mut Board, part: &str) -> FenResult {
    if_chain! {
        if part.len() == 1;
        if let Some(c) = part.chars().next();
        if WHITE_OR_BLACK.contains(c);
        then {
            match c {
                'w' => board.game_state.active_color = Sides::WHITE as u8,
                'b' => board.game_state.active_color = Sides::BLACK as u8,
                _ => (),
            }
            return Ok(());
        }
    }

    Err(FenError::Part2)
}

// Part 3: Parse castling rights.
fn castling(board: &mut Board, part: &str) -> FenResult {
    // There should be 1 to 4 castling rights. If no player has castling
    // rights, the character is '-'.
    if (1..=4).contains(&part.len()) {
        // Accepts "-" for no castling rights in addition to leaving out letters.
        for c in part.chars() {
            match c {
                'K' => board.game_state.castling |= Castling::WK,
                'Q' => board.game_state.castling |= Castling::WQ,
                'k' => board.game_state.castling |= Castling::BK,
                'q' => board.game_state.castling |= Castling::BQ,
                '-' => (),
                _ => return Err(FenError::Part3),
            }
        }
        return Ok(());
    }

    Err(FenError::Part3)
}

// Part 4: Parse the en passant square
fn en_passant(board: &mut Board, part: &str) -> FenResult {
    // No en-passant square if length is 1. The character should be a DASH.
    if_chain! {
        if part.len() == 1;
        if let Some(x) = part.chars().next();
        if x == DASH;
        then {
            return Ok(());
        }
    }

    // If length is 2, try to parse the part to a square number.
    if part.len() == 2 {
        let square = parse::algebraic_square_to_number(part);

        match square {
            Some(sq) if EP_SQUARES_WHITE.contains(&sq) || EP_SQUARES_BLACK.contains(&sq) => {
                board.game_state.en_passant = Some(sq as u8);
                return Ok(());
            }
            _ => return Err(FenError::Part4),
        };
    }

    Err(FenError::Part4)
}

// Part 5: Half-move clock: parse number of moves since last capture or pawn push.
fn half_move_clock(board: &mut Board, part: &str) -> FenResult {
    if_chain! {
        if part.len() == 1 || part.len() == 2;
        if let Ok(x) = part.parse::<u8>();
        if x <= MAX_MOVE_RULE;
        then {
            board.game_state.halfmove_clock = x;
            return Ok(());
        }
    }

    Err(FenError::Part5)
}

// Part 6: Parse full move number.
fn full_move_number(board: &mut Board, part: &str) -> FenResult {
    if_chain! {
        if !part.is_empty() && part.len() <= 4;
        if let Ok(x) = part.parse::<u16>();
        if x <= (MAX_GAME_MOVES as u16);
        then {
            board.game_state.fullmove_number = x;
            return Ok(());
        }
    }

    Err(FenError::Part6)
}
