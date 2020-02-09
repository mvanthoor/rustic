use crate::board::Board;
use crate::defines::*;
use crate::magics::Magics;

pub struct Move {
    data: usize,
    score: u32,
}

fn next(bitboard: &mut Bitboard) -> u8 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1 << location;
    (location as u8)
}

fn king(board: &Board, side: Side, moves: &mut Vec<Move>) {
    println!("King");
    let mut bitboard = board.piece(KING, side);
    println!("Before: {:b}", bitboard);
    let location = next(&mut bitboard);
    println!("King location: {}", SQUARE_NAME[location as usize]);
    println!("After: {:b}", bitboard);
}

fn knight(board: &Board, side: Side) {
    println!("Knight");
    let mut bitboard = board.piece(KNIGHT, side);
    while bitboard > 0 {
        println!("Before: {:b}", bitboard);
        let location = next(&mut bitboard);
        println!("Knight location: {}", SQUARE_NAME[location as usize]);
        println!("After: {:b}", bitboard);
    }
}

fn pawns(board: &Board, side: Side) {
    println!("Pawns");
    let mut bitboard = board.piece(PAWN, side);
    while bitboard > 0 {
        println!("Before: {:b}", bitboard);
        let location = next(&mut bitboard);
        println!("Pawn location: {}", SQUARE_NAME[location as usize]);
        println!("After: {:b}", bitboard);
    }
}

pub fn generate(board: &Board, side: Side, magics: &Magics) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::with_capacity(MAX_MOVES as usize);
    println!("Generating moves...");
    king(board, side, &mut moves);
    // knight(board, side);
    // pawns(board, side);
    moves
}
