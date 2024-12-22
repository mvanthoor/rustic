use crate::{
    board::defs::{Pieces, SQUARE_NAME},
    defs::{Piece, Square},
};
use if_chain::if_chain;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ParseMoveError {
    WrongMoveLength,
    CannotConvertSquare,
    CannotConvertPromotionPiece,
}

impl fmt::Display for ParseMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            Self::WrongMoveLength => "Move must be of length 4 or 5",
            Self::CannotConvertSquare => "Cannot convert square to number",
            Self::CannotConvertPromotionPiece => "Cannot convert promotion piece",
        };
        write!(f, "{error}")
    }
}

impl Error for ParseMoveError {}

pub type ConvertedMove = (Square, Square, Piece);
pub type ParseMoveResult = Result<ConvertedMove, ParseMoveError>;

pub fn algebraic_move_to_numbers(m: &str) -> ParseMoveResult {
    let lower_case_move = m.to_ascii_lowercase();
    let mut converted_move: ConvertedMove = (0, 0, Pieces::NONE);

    if m.len() != 4 && m.len() != 5 {
        return Err(ParseMoveError::WrongMoveLength);
    }

    // Get the "from" and "to" squares from the move string.
    if_chain! {
        // If conversion from algebraic square to number succeeds...
        if let Some(from) = algebraic_square_to_number(&lower_case_move[0..=1]);
        if let Some(to) = algebraic_square_to_number(&lower_case_move[2..=3]);
        then {
            // ...save the result
            converted_move.0 = from;
            converted_move.1 = to;
        } else {
            return Err(ParseMoveError::CannotConvertSquare)
        }
    };

    // Keep parsing if there are 5 characters.
    if m.len() == 5 {
        // Get the promotion piece character.
        let c = lower_case_move.chars().last().unwrap_or('-');

        // If the conversion from character to promotion piece succeeds...
        if let Some(piece) = promotion_piece_letter_to_number(c) {
            // ...save the result
            converted_move.2 = piece;
        } else {
            return Err(ParseMoveError::CannotConvertPromotionPiece);
        }
    }

    Ok(converted_move)
}

// Convert square names to numbers.
pub fn algebraic_square_to_number(algebraic_square: &str) -> Option<Square> {
    SQUARE_NAME
        .iter()
        .position(|&element| element == algebraic_square)
}

// Convert promotion piece names to number
pub fn promotion_piece_letter_to_number(piece_letter: char) -> Option<Piece> {
    // Assume that the character does not represent a promotion piece.
    // Note that this is NOT 'no piece' as in Pieces::NONE! This is 'no
    // piece' als in "no piece found for the provided character". This
    // happens if the character is 'a' or 'z' for example.
    let mut piece: Option<Piece> = None;

    // Convert the character to lowercase and get it for processing.
    if let Some(p) = piece_letter.to_lowercase().next() {
        // Set the promotion piece if the character is correct.
        match p {
            'q' => piece = Some(Pieces::QUEEN),
            'r' => piece = Some(Pieces::ROOK),
            'b' => piece = Some(Pieces::BISHOP),
            'n' => piece = Some(Pieces::KNIGHT),
            _ => (),
        }
    }

    // Return the piece if found, or None.
    piece
}
