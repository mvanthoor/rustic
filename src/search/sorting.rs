// Move sorting routines.

use crate::defs::NrOf;

// MVV_VLA[victim][attacker]
pub const MVV_LVA: [[u8; NrOf::PIECE_TYPES]; NrOf::PIECE_TYPES] = [
    [0, 0, 0, 0, 0, 0],       // King, captured by K, Q, R, B, N, P (impossible)
    [50, 51, 52, 53, 54, 55], // Queen, captured by K, Q, R, B, N, P
    [40, 41, 42, 43, 44, 45], // Rook, captured by K, Q, R, B, N, P
    [30, 31, 32, 33, 34, 35], // Bishop, captured by K, Q, R, B, N, P
    [20, 21, 22, 23, 24, 25], // Knight, captured by K, Q, R, B, N, P
    [10, 11, 12, 13, 14, 15], // Pawn, captured by K, Q, R, B, N, P
];
