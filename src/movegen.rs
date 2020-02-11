use crate::board::Board;
use crate::defines::*;
use crate::magics::Magics;

pub const MAX_LEGAL_MOVES: u8 = 255;

enum MoveType {
    Normal,
    Capture,
    Castle,
    EnPassant,
}

pub struct Move {
    data: u64,
    score: u32,
}

pub type MoveList = Vec<Move>;

pub fn generate(board: &Board, side: Side, magics: &Magics, moves: &mut MoveList) {
    non_slider(KING, board, side, magics, moves);
    non_slider(KNIGHT, board, side, magics, moves);
}

fn non_slider(piece: Piece, board: &Board, side: Side, magics: &Magics, moves: &mut MoveList) {
    debug_assert!(piece == KING || piece == KNIGHT, "Not a non-slider piece!");
    let opponent = side ^ 1;
    let mut bitboard = board.piece(piece, side);
    while bitboard > 0 {
        let from = next(&mut bitboard) as usize;
        let mask: Bitboard = match piece {
            KING => magics.king[from],
            KNIGHT => magics.knight[from],
            _ => 0,
        };
        let normal_to = mask & !board.occupancy();
        let capture_to = mask & board.bb_pieces[opponent];
        add_move(from as u64, normal_to, MoveType::Normal, moves);
        add_move(from as u64, capture_to, MoveType::Capture, moves);
    }
}

fn next(bitboard: &mut Bitboard) -> u64 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1 << location;
    location as u64
}

fn add_move(from: u64, to: Bitboard, mtype: MoveType, moves: &mut MoveList) {
    let mut bitboard_to = to;
    match mtype {
        MoveType::Normal => (),
        MoveType::Capture => (),
        _ => (),
    }
    while bitboard_to > 0 {
        let to_square = next(&mut bitboard_to);
        moves.push(Move {
            data: from ^ (to_square << 6),
            score: 0,
        });
    }
}
