// This structure implements Piece Square Tables (PSQT) for each piece.

use crate::{
    board::{defs::Pieces, Board},
    defs::{NrOf, Sides},
    //misc::bits,
};

type PSQT = [i8; NrOf::SQUARES];

#[allow(dead_code)]
#[rustfmt::skip]
const KING_MG: PSQT = [
    -50,   -40,   -30,   -30,   -30,   -30,   -40,   -50,
    -40,   -20,   -20,   -20,   -20,   -20,   -20,   -40,
    -30,   -20,   -20,   -20,   -20,   -20,   -20,   -30,
    -30,   -20,   -20,   -20,   -20,   -20,   -20,   -30,
    -30,   -20,   -20,   -20,   -20,   -20,   -20,   -30,
    -30,   -20,   -20,   -20,   -20,   -20,   -20,   -30,
      5,     5,   -10,   -10,   -10,   -10,     5,     5,
     15,    20,    15,    -5,     0,    -5,    20,    15
];

#[allow(dead_code)]
#[rustfmt::skip]
const QUEEN_MG: PSQT = [
    -30,  -20,  -10,  -10,  -10,  -10,  -20,  -30,
    -20,  -10,   -5,   -5,   -5,   -5,  -10,  -20,
    -10,   -5,   10,   10,   10,   10,   -5,  -10,
    -10,   -5,   10,   20,   20,   10,   -5,  -10,
    -10,   -5,   10,   20,   20,   10,   -5,  -10,
    -10,   -5,   10,   10,   10,   10,   -5,  -10,
    -20,  -10,   -5,   -5,   -5,   -5,  -10,  -20,
    -30,  -20,  -10,  -10,  -10,  -10,  -20,  -30 
];

#[allow(dead_code)]
#[rustfmt::skip]
const ROOK_MG: PSQT = [
    0,   0,   0,   0,   0,   0,   0,   0,
    5,   5,  10,  15,  15,  10,   5,   5,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,  10,  10,   5,   0,   0
];

#[allow(dead_code)]
#[rustfmt::skip]
const BISHOP_MG: PSQT = [
    -20,    0,    0,    0,    0,    0,    0,  -20,
    -15,    0,    0,    0,    0,    0,    0,  -15,
    -10,    0,    0,    5,    5,    0,    0,  -10,
    -10,    5,    5,   10,   10,    5,    5,  -10,
      5,    5,    5,   10,   10,    5,    5,    5,
      5,    5,    5,    5,    5,    5,    5,    5,
    -10,    5,    5,    5,    5,    5,    5,  -10,
    -20,  -10,  -10,  -10,  -10,  -10,  -10,  -20
];

#[allow(dead_code)]
#[rustfmt::skip]
const KNIGHT_MG: PSQT = [
    -20, -10,  -10,  -10,  -10,  -10,  -10,  -20,
    -10,  -5,   -5,   -5,   -5,   -5,   -5,  -10,
    -10,  -5,   25,   25,   25,   25,   -5,  -10,
    -10,  -5,   20,   20,   20,   20,   -5,  -10,
    -10,  -5,   15,   15,   15,   15,   -5,  -10,
    -10,  -5,   10,    5,    5,   10,   -5,  -10,
    -10,  -5,   -5,   -5,   -5,   -5,   -5,  -10,
    -20, -10,  -10,  -10,  -10,  -10,  -10,  -20
];

#[allow(dead_code)]
#[rustfmt::skip]
const PAWN_MG: PSQT = [
     0,   0,   0,   0,   0,   0,   0,   0,
    15,  15,  15,  15,  15,  15,  15,  15,
    10,  10,  10,  10,  10,  10,  10,  10,
     5,   5,  10,  10,  10,   5,   5,   5,
     5,   5,  10,  10,  10,   5,   5,   5,
     5,   5,   5,   5,   5,   0,   5,   5,
     5,   5,   5,  10,  10,   0,   5,   5,
     0,   0,   0,   0,   0,   0,   0,   0
];

const PSQT_MG: [PSQT; NrOf::PIECE_TYPES] =
    [KING_MG, QUEEN_MG, ROOK_MG, BISHOP_MG, KNIGHT_MG, PAWN_MG];

// To make the Piece Square tables easier to relate to, and easier to
// edit, they have been laid out as a normal chess board, with A1 at
// the lower left. Because the square numbers start with A1 = 0, a
// conversion needs to be done. Below are the representations of the
// square numbers and the PSQT from both white and black point of view.

// (These tables will hold piece/square values instead of coordinates.
// The coordinates are used to visualize to which square a value in the
// table would belong.)

// Square numbers, as they are in arrays:

//  0  1  2  3  4  5  6  7   <= 0 = A1, 7 = H1
//  8  9 10 11 12 13 14 15
// 16 17 18 19 20 21 22 23
// 24 25 26 27 28 29 30 31
// 32 33 34 35 36 37 38 39
// 40 41 42 43 44 45 46 47
// 48 49 50 51 52 53 54 55
// 56 57 58 59 60 61 62 63  <= 56 = A8, 63 = H8

// PSQT, WHITE:                // Same PSQT, BLACK:

// A8 B8 C8 D8 E8 F8 G8 H8  |  // A1 B1 C1 D1 E1 G1 F1 H1
// A7 B7 C7 D8 E8 F8 G7 H7  |  // A2 B2 C2 D2 E2 G2 F2 H2
// A6 B6 C6 D6 E6 F6 G6 H6  |  // A3 B3 C3 D3 E3 G3 F3 H3
// A5 B5 C5 D5 E5 F5 G5 H5  |  // A4 B4 C4 D4 E4 G4 F4 H4
// A4 B4 C4 D4 E4 F4 G4 H4  |  // A5 B5 C5 D5 E5 G5 F5 H5
// A3 B3 C3 D3 E3 F3 G3 H3  |  // A6 B6 C6 D6 E6 G6 F6 H6
// A2 B2 C2 D2 E2 F2 G2 H2  |  // A7 B7 C7 D7 E7 G7 F7 H7
// A1 B1 C1 D1 E1 F1 G1 H1  |  // A8 B8 C8 D8 E8 F8 G8 H8

// If one super-imposes the square numbers on the PSQT with square names
// from WHITE's point of view, it can be seen that the following is true:

/*
    Square to PSQT element examples:
    A1 = square 0.  ==> PSQT element 56.
    H8 = square 63. ==> PSQT element 7.
    E8 = square 60. ==> PSQT element 4.
    E1 = square 4.  ==> PSQT element 60.
*/

// One can also see that, if the SAME PSQT from WHITE's point of view is to
// be used for BLACK, it can be indexed by the square number, without any
// converstion. (Super-impose the square numbers on top of the PSQT with
// BLACK square names.)

// This results in the following converstion table, from aquare number
// to PSQT element, needed for WHITE only::

#[allow(dead_code)]
#[rustfmt::skip]
const FLIP: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63,
    48, 49, 50, 51, 52, 53, 54, 55,
    40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39,
    24, 25, 26, 27, 28, 29, 30, 31,
    16, 17, 18, 19, 20, 21, 22, 23,
     8,  9, 10, 11, 12, 13, 14, 15,
     0,  1,  2,  3,  4,  5,  6,  7,
];

// Apply PSQT's to position
pub fn apply(board: &Board) -> i16 {
    let w_ks = board.king_square(Sides::WHITE);
    let b_ks = board.king_square(Sides::BLACK);
    let wk_value = PSQT_MG[Pieces::KING][FLIP[w_ks]];
    let bk_value = PSQT_MG[Pieces::KING][b_ks];

    let adjustment = (wk_value - bk_value) as i16;

    println!("Adjustment for wK: {}", wk_value);
    println!("Adjustment for bK: {}", bk_value);
    println!("King adjustment: {}", adjustment);

    adjustment
}
