use crate::board::Board;
use crate::defines::*;
use crate::magics::Magics;

pub struct Move {
    data: u64,
    score: u32,
}
pub type MoveList = Vec<Move>;

fn next(bitboard: &mut Bitboard) -> u64 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1 << location;
    location as u64
}

fn add_move(from: u64, to: Bitboard, mtype: MoveType, moves: &mut MoveList) {
    let mut bitboard_to = to;
    match mtype {
        MoveType::Quiet => println!("Quiet"),
        MoveType::Capture => println!("Capture"),
    }
    while bitboard_to > 0 {
        let to_square = next(&mut bitboard_to);
        moves.push(Move {
            data: from ^ (to_square << 6),
            score: 0,
        });
        println!(
            "{}{} - from {:b}, to {:b}, data: {:b}",
            SQUARE_NAME[from as usize],
            SQUARE_NAME[to_square as usize],
            from,
            to_square,
            from ^ (to_square << 6)
        )
    }
}

fn non_slider(piece: Piece, board: &Board, side: Side, magics: &Magics, moves: &mut Vec<Move>) {
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
        let quiet_to = mask & !board.occupancy();
        let capture_to = mask & board.bb_pieces[opponent];
        add_move(from as u64, quiet_to, MoveType::Quiet, moves);
        add_move(from as u64, capture_to, MoveType::Capture, moves);
    }
}

// fn pawns(board: &Board, side: Side) {
//     println!("Pawns");
//     let mut bitboard = board.piece(PAWN, side);
//     while bitboard > 0 {
//         println!("Before: {:b}", bitboard);
//         let location = next(&mut bitboard);
//         println!("Pawn location: {}", SQUARE_NAME[location as usize]);
//         println!("After: {:b}", bitboard);
//     }
// }

pub fn generate(board: &Board, side: Side, magics: &Magics) -> MoveList {
    let mut moves: MoveList = Vec::with_capacity(MOVE_MAX as usize);
    println!("Generating moves...");
    println!("King");
    non_slider(KING, board, side, magics, &mut moves);
    println!("Knight");
    non_slider(KNIGHT, board, side, magics, &mut moves);
    moves
}
