// This file implements Piece Square Tables (PST) for each piece type. The
// PST's are written from White's point of view, as if looking at a chess
// diagram, with A1 on the lower left corner.

use crate::{
    board::Board,
    defs::{NrOf, Sides},
    evaluation::defs::{EvalParams, W},
    evaluation::Evaluation,
    misc::bits,
};

pub type FlipTable = [usize; NrOf::SQUARES];
pub type Psqt = [W; NrOf::SQUARES];
pub type PsqtSet = [Psqt; NrOf::PIECE_TYPES];

#[rustfmt::skip]
const PSQT_KING: Psqt = 
[
    W(0,-95), W(0,-95), W( 0,-90), W(  0,-90), W(  0,-90), W(0,-90), W( 0,-95), W(0,-95),
    W(0,-95), W(0,-50), W( 0,-50), W(  0,-50), W(  0,-50), W(0,-50), W( 0,-50), W(0,-95),
    W(0,-90), W(0,-50), W( 0,-20), W(  0,-20), W(  0,-20), W(0,-20), W( 0,-50), W(0,-90),
    W(0,-90), W(0,-50), W( 0,-20), W(  0,  0), W(  0,  0), W(0,-20), W( 0,-50), W(0,-90),
    W(0,-90), W(0,-50), W( 0,-20), W(  0,  0), W(  0,  0), W(0,-20), W( 0,-50), W(0,-90),
    W(0,-90), W(0,-50), W( 0,-20), W(  0,-20), W(  0,-20), W(0,-20), W( 0,-50), W(0,-90),
    W(0,-95), W(0,-50), W( 0,-50), W(-10,-50), W(-10,-50), W(0,-50), W( 0,-50), W(0,-95),
    W(0,-95), W(0,-95), W(20,-90), W(-10,-90), W(-10,-90), W(0,-90), W(20,-95), W(0,-95),
];

#[rustfmt::skip]
const PSQT_QUEEN: Psqt = 
[
    W(870,870), W(880,880), W(890,890), W(890,890), W(890,890), W(890,890), W(880,880), W(870,870),
    W(880,880), W(890,890), W(895,895), W(895,895), W(895,895), W(895,895), W(890,890), W(880,880),
    W(890,890), W(895,895), W(910,910), W(910,910), W(910,910), W(910,910), W(895,895), W(890,890),
    W(890,890), W(895,895), W(910,910), W(920,920), W(920,920), W(910,910), W(895,895), W(890,890),
    W(890,890), W(895,895), W(910,910), W(920,920), W(920,920), W(910,910), W(895,895), W(890,890),
    W(890,890), W(895,895), W(895,895), W(895,895), W(895,895), W(895,895), W(895,895), W(890,890),
    W(880,880), W(890,890), W(895,895), W(895,895), W(895,895), W(895,895), W(890,890), W(880,880),
    W(870,870), W(880,880), W(890,890), W(890,890), W(890,890), W(890,890), W(880,880), W(870,870),
];

#[rustfmt::skip]
const PSQT_ROOK: Psqt = 
[
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(515,515), W(515,515), W(515,515), W(520,520), W(520,520), W(515,515), W(515,515), W(515,515),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(510,510), W(510,510), W(510,510), W(500,500), W(500,500),
];

#[rustfmt::skip]
const PSQT_BISHOP: Psqt = 
[
    W(300,300), W(320,320), W(320,320), W(320,320), W(320,320), W(320,320), W(320,320), W(300,300),
    W(305,305), W(320,320), W(320,320), W(320,320), W(320,320), W(320,320), W(320,320), W(305,305),
    W(310,310), W(320,320), W(320,320), W(325,325), W(325,325), W(320,320), W(320,320), W(310,310),
    W(310,310), W(330,330), W(330,330), W(350,350), W(350,350), W(330,330), W(330,330), W(310,310),
    W(325,325), W(325,325), W(330,330), W(345,345), W(345,345), W(330,330), W(325,325), W(325,325),
    W(325,325), W(325,325), W(325,325), W(330,330), W(330,330), W(325,325), W(325,325), W(325,325),
    W(310,310), W(325,325), W(325,325), W(330,330), W(330,330), W(325,325), W(325,325), W(310,310),
    W(300,300), W(310,310), W(310,310), W(310,310), W(310,310), W(310,310), W(310,310), W(300,300),
];

#[rustfmt::skip]
const PSQT_KNIGHT: Psqt =     
[
    W(290,290), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(290,290),
    W(300,300), W(305,305), W(305,305), W(305,305), W(305,305), W(305,305), W(305,305), W(300,300),
    W(300,300), W(305,305), W(325,325), W(325,325), W(325,325), W(325,325), W(305,305), W(300,300),
    W(300,300), W(305,305), W(325,325), W(325,325), W(325,325), W(325,325), W(305,305), W(300,300),
    W(300,300), W(305,305), W(325,325), W(325,325), W(325,325), W(325,325), W(305,305), W(300,300),
    W(300,300), W(305,305), W(320,320), W(325,325), W(325,325), W(325,325), W(305,305), W(300,300),
    W(300,300), W(305,305), W(305,305), W(305,305), W(305,305), W(305,305), W(305,305), W(300,300),
    W(290,290), W(310,310), W(300,300), W(300,300), W(300,300), W(300,300), W(310,310), W(290,290),
];

#[rustfmt::skip]
const PSQT_PAWN: Psqt = 
[
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
    W(160,160), W(160,160), W(160,160), W(160,160), W(170,170), W(160,160), W(160,160), W(160,160),
    W(140,140), W(140,140), W(140,140), W(150,150), W(160,160), W(140,140), W(140,140), W(140,140),
    W(120,120), W(120,120), W(120,120), W(140,140), W(150,150), W(120,120), W(120,120), W(120,120),
    W(105,105), W(105,105), W(115,115), W(130,130), W(140,140), W(110,110), W(105,105), W(105,105),
    W(105,105), W(105,105), W(110,110), W(120,120), W(130,130), W(105,105), W(105,105), W(105,105),
    W(105,105), W(105,105), W(105,105), W( 70, 70), W( 70, 70), W(105,105), W(105,105), W(105,105),
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
];

pub const PSQT_SET: PsqtSet = [
    PSQT_KING,
    PSQT_QUEEN,
    PSQT_ROOK,
    PSQT_BISHOP,
    PSQT_KNIGHT,
    PSQT_PAWN,
];

// To make the Piece Square tables easier to relate to, and easier to
// edit, they have been laid out as a normal chess board, with A1 at
// the lower left. Because the square numbers start with A1 = 0, a
// conversion needs to be done. Below are the representations of the
// square numbers and the PST from both white and black point of view.

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

// PST, WHITE:                // Same PST, BLACK:

// A8 B8 C8 D8 E8 F8 G8 H8  |  // A1 B1 C1 D1 E1 G1 F1 H1
// A7 B7 C7 D8 E8 F8 G7 H7  |  // A2 B2 C2 D2 E2 G2 F2 H2
// A6 B6 C6 D6 E6 F6 G6 H6  |  // A3 B3 C3 D3 E3 G3 F3 H3
// A5 B5 C5 D5 E5 F5 G5 H5  |  // A4 B4 C4 D4 E4 G4 F4 H4
// A4 B4 C4 D4 E4 F4 G4 H4  |  // A5 B5 C5 D5 E5 G5 F5 H5
// A3 B3 C3 D3 E3 F3 G3 H3  |  // A6 B6 C6 D6 E6 F6 G6 H6
// A2 B2 C2 D2 E2 F2 G2 H2  |  // A7 B7 C7 D7 E7 G7 F7 H7
// A1 B1 C1 D1 E1 F1 G1 H1  |  // A8 B8 C8 D8 E8 F8 G8 H8

// If one super-imposes the square numbers on the PST with square names
// from WHITE's point of view, it can be seen that the following is true:

/*
    Square to PST element examples:
    A1 = square 0.  ==> PST element 56.
    H8 = square 63. ==> PST element 7.
    E8 = square 60. ==> PST element 4.
    E1 = square 4.  ==> PST element 60.
*/

// One can also see that, if the SAME PST from WHITE's point of view is to
// be used for BLACK, it can be indexed by the square number, without any
// conversation. (Super-impose the square numbers on top of the PST with
// BLACK square names.)

// This results in the following conversation table, from Square number
// to PST element, needed for WHITE only::

#[allow(dead_code)]
#[rustfmt::skip]
pub const FLIP: FlipTable = [
    56, 57, 58, 59, 60, 61, 62, 63,
    48, 49, 50, 51, 52, 53, 54, 55,
    40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39,
    24, 25, 26, 27, 28, 29, 30, 31,
    16, 17, 18, 19, 20, 21, 22, 23,
     8,  9, 10, 11, 12, 13, 14, 15,
     0,  1,  2,  3,  4,  5,  6,  7,
];

impl Evaluation {
    // Apply the PSQT's to the current position. This will only be done
    // during board initialization, after a position has been loaded by the
    // FEN-reader. The outcome will be placed in the board's game state, so
    // the engine can update the values incrementally very time a piece is
    // moved.
    pub fn psqt_apply(board: &Board, psqt_set: &PsqtSet) -> (W, W) {
        let mut psqt_w_mg: i16 = 0;
        let mut psqt_w_eg: i16 = 0;
        let mut psqt_b_mg: i16 = 0;
        let mut psqt_b_eg: i16 = 0;
        let bb_white = board.bb_pieces[Sides::WHITE]; // Array of white piece bitboards
        let bb_black = board.bb_pieces[Sides::BLACK]; // Array of black piece bitboards

        // Iterate through the white and black bitboards (at the same time.)
        for (piece_type, (w, b)) in bb_white.iter().zip(bb_black.iter()).enumerate() {
            let mut white_pieces = *w; // White pieces of type "piece_type"
            let mut black_pieces = *b; // Black pieces of type "piece_type"

            // Iterate over pieces of the current piece_type for white.
            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                psqt_w_mg += psqt_set[piece_type][FLIP[square]].mg();
                psqt_w_eg += psqt_set[piece_type][FLIP[square]].eg();
            }

            // Iterate over pieces of the current piece_type for black.
            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                psqt_b_mg += psqt_set[piece_type][square].mg();
                psqt_b_eg += psqt_set[piece_type][square].eg()
            }
        }

        (W(psqt_w_mg, psqt_w_eg), W(psqt_b_mg, psqt_b_eg))
    }

    // Interpolate PST values between midgame and endgame tables. This
    // makes the engine much stronger, because it can now take into account
    // that piece values and locations are different in the opening/midgame
    // and endgame.
    pub fn psqt_score(board: &Board) -> i16 {
        // Get current PST values. These are kept incrementally during play.
        let psqt_w_mg = board.game_state.psqt_value[Sides::WHITE].mg() as f32;
        let psqt_b_mg = board.game_state.psqt_value[Sides::BLACK].mg() as f32;
        let psqt_w_eg = board.game_state.psqt_value[Sides::WHITE].eg() as f32;
        let psqt_b_eg = board.game_state.psqt_value[Sides::BLACK].eg() as f32;

        // Get the game phase, from 1 (opening/midgame) to 0 (endgame)
        let v = board.game_state.phase_value;
        let phase = Evaluation::determine_phase(EvalParams::PHASE_MIN, EvalParams::PHASE_MAX, v);

        // Mix the tables by taking parts of both mg and eg.
        let score_w = (psqt_w_mg * phase) + (psqt_w_eg * (1.0 - phase));
        let score_b = (psqt_b_mg * phase) + (psqt_b_eg * (1.0 - phase));

        // Return final PST score.
        (score_w - score_b).round() as i16
    }
}
