use crate::{
    board::defs::{Pieces, SQUARE_NAME},
    defs::{Piece, Square},
};
use if_chain::if_chain;

pub type PotentialMove = (Square, Square, Piece);
pub type ParseMoveResult = Result<PotentialMove, ()>;

pub fn algebraic_move_to_square_numbers(m: &str) -> ParseMoveResult {
    let lower_case_move = m.to_ascii_lowercase();
    let mut potential_move: PotentialMove = (0, 0, Pieces::NONE);

    // Assume parsing the move will fail.
    let mut parse_move_result: ParseMoveResult = Err(());

    // Get the "from" and "to" squares from the move string.
    if m.len() == 4 || m.len() == 5 {
        if_chain! {
            // If conversion from algebraic square to number succeeds...
            if let Some(f) = algebraic_square_to_number(&lower_case_move[0..=1]);
            if let Some(t) = algebraic_square_to_number(&lower_case_move[2..=3]);
            then {
                // ...save the result
                potential_move.0 = f;
                potential_move.1 = t;

                // Up to here, parsing is OK.
                parse_move_result = Ok(potential_move);
            }
        };
    }

    // If Ok and there are 5 characters, keep parsing...
    if parse_move_result != Err(()) && m.len() == 5 {
        // Again, assume that parsing will fail.
        parse_move_result = Err(());

        // Get the promotion piece character.
        let c = lower_case_move.chars().last().unwrap_or('-');

        // If the conversion from character to promotion piece succeeds...
        if let Some(p) = promotion_piece_letter_to_number(c) {
            // ...save the result
            potential_move.2 = p;

            // and set the parsing result to Ok again.
            parse_move_result = Ok(potential_move);
        }
    }

    parse_move_result
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
